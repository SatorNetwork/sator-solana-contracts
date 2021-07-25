//! Program owned state
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::clock::UnixTimestamp;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;
use solana_program::{entrypoint::ProgramResult, program_error::ProgramError};
use solana_program::{system_program, sysvar};

use crate::sdk::program::PubkeyPatterns;
use crate::sdk::types::*;
use crate::{program_id, state, types::*};

#[derive(Debug, BorshDeserialize, BorshSerialize, BorshSchema)]
pub struct InitializeStakePoolInput {
    pub ranks: [Rank; 4],
}

#[derive(Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub struct StakeInput {
    /// any of times from [crate::state::ViewerStake::ranks] or more
    pub duration: ApproximateSeconds,
    pub amount: TokenAmount,
}


#[derive(Debug, BorshDeserialize, BorshSerialize, BorshSchema)]
pub enum Instruction {
    InitializeStakePool(InitializeStakePoolInput),
    Stake(StakeInput),
    Unstake,
}

/// Creates [Instruction::InitializeStake] instruction which initializes `stake_pool` and `token_account`
///
/// Accounts:
///  * `system_program`  - *program, implicit* to create accounts
///  * `sysvar_rent`     - *program, implicit* ensure that `token_account` and  `stake_pool` are rent exempt.
///  * `spl_token`       - *program, implicit* spl token program to initialize `token_account`.
///  * `owner`           - *signer, payer* and owner of `stake_pool`.
///  * `stake_pool`       - *mutable, signer* not initialized not created account for stake data.
///  * `stake_authority` - *implicit* program derived account from `32 bytes stake public key` based `program_id`.
///  * `token_account`   - *implicit, mutable, derived* not created program derived account to create `spl_token`  under `stake_authority`.
///
#[allow(clippy::too_many_arguments)]
pub fn initialize_stake(
    owner: &SignerPubkey,
    stake: &SignerPubkey,
    mint: &MintPubkey,
    input: InitializeStakePoolInput,
) -> Result<solana_program::instruction::Instruction, ProgramError> {
    let stake_authority = Pubkey::find_program_address_for_pubkey(stake, &program_id());
    let token_account = Pubkey::create_with_seed(
        &stake_authority.0,
        "ViewerStakePool::token_account",
        &spl_token::id(),
    )?;
    Ok(solana_program::instruction::Instruction::new_with_borsh(
        crate::id(),
        &Instruction::InitializeStakePool(input),
        vec![
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(*owner, true),
            AccountMeta::new(*stake, true),
            AccountMeta::new_readonly(stake_authority.0, false),
            AccountMeta::new(token_account, false),
            AccountMeta::new_readonly(*mint, false),
        ],
    ))
}

/// Creates [Instruction::Stake] instruction which transfer `amount` from `token_account_source` to `token_account_stake_target`.
/// If `stake_account` initialized, resets timer.
///
/// Accounts:
///  * `system_program`             - *program, implicit*
///  * `sysvar_rent`                - *program, implicit* to create `stake_account` which will be rent except if needed
///  * `clock`                      - *program, implicit*
///  * `spl_token`                  - *program, implicit*
///  * `wallet`                     - *signer, payer*
///  * `stake_pool`                 -  
///  * `stake_authority`            - derived  as in [Instruction::InitializeStake]
///  * `token_account_source`       - *mutable*
///  * `token_account_stake_target` - *derived, mutable, implicit*
///  * `stake_account`               - *implicit, derived, mutable* from `wallet` and `stake_authority`
///
/// Notes:
/// - current design does not creates token account to stake tokens, just counts amount in stake.
/// - stake instruction is same instruction as initialize stake, so it could be made different by having separate stake (it will reduce amount of accounts during stake invocation)
#[allow(clippy::too_many_arguments)]
pub fn stake(
    wallet: &SignerPubkey,
    stake_pool: &Pubkey,
    token_account_source: &TokenAccountPubkey,
    input: StakeInput,
) -> Result<(solana_program::instruction::Instruction, Pubkey), ProgramError> {
    let (stake_authority, _) = Pubkey::find_program_address_for_pubkey(stake_pool, &program_id());
    let token_account_stake_target = Pubkey::create_with_seed(
        &stake_authority,
        "ViewerStakePool::token_account",
        &spl_token::id(),
    )?;
    let stake_account =
        Pubkey::create_with_seed_for_pubkey(&stake_authority, wallet, &program_id())?;
    Ok((
        solana_program::instruction::Instruction::new_with_borsh(
            crate::id(),
            &Instruction::Stake(input),
            vec![
                AccountMeta::new_readonly(system_program::id(), false),
                AccountMeta::new_readonly(sysvar::rent::id(), false),
                AccountMeta::new_readonly(sysvar::clock::id(), false),
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly(*wallet, true),
                AccountMeta::new_readonly(*stake_pool, false),
                AccountMeta::new_readonly(stake_authority, false),
                AccountMeta::new(*token_account_source, false),
                AccountMeta::new(token_account_stake_target, false),
                AccountMeta::new(stake_account.0, false),
            ],
        ),
        stake_account.0,
    ))
}

/// Creates [Instruction::Unstake] instruction which transfer `amount` from `token_account_stake_source` to `token_account_target` if and only if now is more than [crate::state::ViewerLock::Staked_until]
/// Resets unlock
///
/// Accounts:
///  * `clock`                      - *program, implicit*
///  * `spl_token`                  - *program, implicit*
///  * `wallet`                     - *signer, payer*
///  * `stake_pool`                 -  
///  * `stake_authority`            - *implicit*, derived from `owner`
///  * `token_account_target`       - *mutable*
///  * `token_account_stake_source` - *derived, mutable, implicit*
///  * `stake_account`               - *implicit, derived, mutable* from `stake_authority` and `wallet`
pub fn unstake(
    wallet: &SignerPubkey,
    stake_pool: &Pubkey,
    token_account_target: &TokenAccountPubkey,
) -> Result<solana_program::instruction::Instruction, ProgramError> {
    let (stake_authority, _) = Pubkey::find_program_address_for_pubkey(stake_pool, &program_id());
    let token_account_stake_source = Pubkey::create_with_seed(
        &stake_authority,
        "ViewerStakePool::token_account",
        &spl_token::id(),
    )?;
    let stake_account =
        Pubkey::create_with_seed_for_pubkey(&stake_authority, wallet, &program_id())?;
    Ok(solana_program::instruction::Instruction::new_with_borsh(
        crate::id(),
        &Instruction::Unstake,
        vec![
            AccountMeta::new_readonly(sysvar::clock::id(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(*wallet, true),
            AccountMeta::new_readonly(*stake_pool, false),
            AccountMeta::new_readonly(stake_authority, false),
            AccountMeta::new(*token_account_target, false),
            AccountMeta::new(token_account_stake_source, false),
            AccountMeta::new(stake_account.0, false),
        ],
    ))
}
