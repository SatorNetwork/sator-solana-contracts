//! Program owned state

use std::time::Duration;

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
pub struct InitializeStakeInput {
    pub ranks: [Ranks; 4],
}

#[derive(Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub struct LockInput {
    /// any of times from [crate::state::ViewerStake::ranks] or more
    pub duration: ApproximateSeconds,
    pub amount: TokenAmount,
}

#[derive(Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub struct UnlockInput {
    pub amount: TokenAmount,
}

#[derive(Debug, BorshDeserialize, BorshSerialize, BorshSchema)]
pub enum Instruction {
    InitializeStake(InitializeStakeInput),
    Lock(LockInput),
    Unlock,
}

/// Creates [Instruction::InitializeStake] instruction which initializes `stake` and `token_account`
///
/// Accounts:
///  * `system_program`  - *program, implicit* to create accounts
///  * `sysvar_rent`     - *program, implicit* ensure that `token_account` and  `stake` are rent exempt.
///  * `spl_token`       - *program, implicit* spl token program to initialize `token_account`.
///  * `owner`           - *signer, payer* and owner of `stake`.
///  * `stake`           - *mutable, signer* not initialized not created account for stake data.
///  * `stake_authority` - *implicit* program derived account from `32 bytes stake public key` based `program_id`.
///  * `token_account`   - *implicit, mutable, derived* not created program derived account to create `spl_token`  under `stake_authority`.
///
#[allow(clippy::too_many_arguments)]
pub fn initialize_stake(
    owner: &SignerPubkey,
    stake: &SignerPubkey,
    mint: &MintPubkey,
    input: InitializeStakeInput,
) -> Result<solana_program::instruction::Instruction, ProgramError> {
    let stake_authority = Pubkey::find_program_address_for_pubkey(stake, &program_id());
    let token_account = Pubkey::create_with_seed(
        &stake_authority.0,
        "ViewerStake::token_account",
        &spl_token::id(),
    )?;
    Ok(solana_program::instruction::Instruction::new_with_borsh(
        crate::id(),
        &Instruction::InitializeStake(input),
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

/// Creates [Instruction::Lock] instruction which transfer `amount` from `token_account_source` to `token_account_stake_target`.
/// If `lock_account` initialized, resets timer.
///
/// Accounts:
///  * `system_program`             - *program, implicit*
///  * `sysvar_rent`                - *program, implicit* to create `lock_account` which will be rent except if needed
///  * `clock`                      - *program, implicit*
///  * `spl_token`                  - *program, implicit*
///  * `wallet`                     - *signer, payer*
///  * `stake`                      -  
///  * `stake_authority`            - derived  as in [Instruction::InitializeStake]
///  * `token_account_source`       - *mutable*
///  * `token_account_stake_target` - *derived, mutable, implicit*
///  * `lock_account`               - *implicit, derived, mutable* from `wallet` and `stake_authority`
///
/// Notes:
/// - current design does not creates token account to lock tokens, just counts amount in lock.
/// - lock instruction is same instruction as initialize lock, so it could be made different by having separate lock (it will reduce amount of accounts during lock invocation)
#[allow(clippy::too_many_arguments)]
pub fn lock(
    wallet: &SignerPubkey,
    stake: &Pubkey,
    token_account_source: &TokenAccountPubkey,
    input: LockInput,
) -> Result<(solana_program::instruction::Instruction, Pubkey), ProgramError> {
    let (stake_authority, _) = Pubkey::find_program_address_for_pubkey(stake, &program_id());
    let token_account_stake_target = Pubkey::create_with_seed(
        &stake_authority,
        "ViewerStake::token_account",
        &spl_token::id(),
    )?;
    let lock_account =
        Pubkey::create_with_seed_for_pubkey(&stake_authority, wallet, &program_id())?;
    Ok((
        solana_program::instruction::Instruction::new_with_borsh(
            crate::id(),
            &Instruction::Lock(input),
            vec![
                AccountMeta::new_readonly(system_program::id(), false),
                AccountMeta::new_readonly(sysvar::rent::id(), false),
                AccountMeta::new_readonly(sysvar::clock::id(), false),
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly(*wallet, true),
                AccountMeta::new_readonly(*stake, false),
                AccountMeta::new_readonly(stake_authority, false),
                AccountMeta::new(*token_account_source, false),
                AccountMeta::new(token_account_stake_target, false),
                AccountMeta::new(lock_account.0, false),
            ],
        ),
        lock_account.0,
    ))
}

/// Creates [Instruction::Unlock] instruction which transfer `amount` from `token_account_stake_source` to `token_account_target` if and only if now is more than [crate::state::ViewerLock::locked_until]
/// Resets unlock
///
/// Accounts:
///  * `clock`                      - *program, implicit*
///  * `spl_token`                  - *program, implicit*
///  * `wallet`                     - *signer, payer*
///  * `stake`                      -  
///  * `stake_authority`            - *implicit*, derived from `owner`
///  * `token_account_target`       - *mutable*
///  * `token_account_stake_source` - *derived, mutable, implicit*
///  * `lock_account`               - *implicit, derived, mutable* from `stake_authority` and `wallet`
#[allow(clippy::too_many_arguments)]
pub fn unlock(
    wallet: &SignerPubkey,
    stake: &Pubkey,
    token_account_target: &TokenAccountPubkey,
) -> Result<solana_program::instruction::Instruction, ProgramError> {
    let (stake_authority, _) = Pubkey::find_program_address_for_pubkey(stake, &program_id());
    let token_account_stake_source = Pubkey::create_with_seed(
        &stake_authority,
        "ViewerStake::token_account",
        &spl_token::id(),
    )?;
    let lock_account =
        Pubkey::create_with_seed_for_pubkey(&stake_authority, wallet, &program_id())?;
    Ok(solana_program::instruction::Instruction::new_with_borsh(
        crate::id(),
        &Instruction::Unlock,
        vec![
            AccountMeta::new_readonly(sysvar::clock::id(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(*wallet, true),
            AccountMeta::new_readonly(*stake, false),
            AccountMeta::new_readonly(stake_authority, false),
            AccountMeta::new(*token_account_target, false),
            AccountMeta::new(token_account_stake_source, false),
            AccountMeta::new(lock_account.0, false),
        ],
    ))
}
