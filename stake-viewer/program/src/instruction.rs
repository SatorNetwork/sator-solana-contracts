//! Program instruction state
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use sator_sdk::program::PubkeyPatterns;
use sator_sdk::types::{
    ApproximateSeconds, MintPubkey, SignerPubkey, TokenAccountPubkey, TokenAmount, PossiblySignerPubkey,
};
use solana_program::instruction::AccountMeta;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_program::{system_program, sysvar};

use crate::{stake_viewer_program_id, types::*};

#[derive(Debug, BorshDeserialize, BorshSerialize, BorshSchema)]
pub struct InitializeStakePoolInput {
    pub ranks: [Rank; 4],
}

#[derive(Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub struct StakeInput {
    /// any of times from [crate::state::ViewerStakePool::ranks] or more
    pub duration: ApproximateSeconds,
    pub amount: TokenAmount,
}

#[derive(Debug, BorshDeserialize, BorshSerialize, BorshSchema)]
pub enum Instruction {
    ///Like: 0u8[(i64,u64),(i64,u64),(i64,u64),(i64,u64)]
    InitializeStakePool(InitializeStakePoolInput),
    ///Like: 1u8(i64,u64)
    Stake(StakeInput),
    //Like: 2u8
    Unstake,
}

/// Creates [Instruction::InitializeStakePool] instruction which initializes `stake_pool` and `token_account`
///
/// Accounts:
///  * `system_program`           - *program, implicit* to create accounts
///  * `sysvar_rent`              - *program, implicit* ensure that `token_account` and  `stake_pool` are rent exempt.
///  * `spl_token`                - *program, implicit* spl token program to initialize `token_account`.
///  * `fee_payer`                -  *signer* pays for account creation
///  * `stake_pool_owner`         - *signer, payer* and owner of `stake_pool`.
///  * `stake_pool`               - *mutable, signer* not initialized not created account for stake data.
///  * `stake_authority`          - *implicit, derived* program derived account from `32 bytes stake public key` based `program_id`.
///  * `token_account`            - *implicit, mutable, derived* not created program derived account to create `spl_token`  under `stake_authority`.
///  * `token_account_stake_pool` - used to initialize `token_account` for reference
#[allow(clippy::too_many_arguments)]
pub fn initialize_stake_pool(
    fee_payer: &SignerPubkey,
    stake_pool_owner: &SignerPubkey,
    stake_pool: &SignerPubkey,
    mint: &MintPubkey,
    input: InitializeStakePoolInput,
) -> Result<solana_program::instruction::Instruction, ProgramError> {
    let stake_authority =
        Pubkey::find_program_address_for_pubkey(stake_pool, &stake_viewer_program_id());
    let token_account_stake_pool = Pubkey::create_with_seed(
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
            AccountMeta::new_readonly(*fee_payer, true),
            AccountMeta::new_readonly(*stake_pool_owner, true),
            AccountMeta::new(*stake_pool, true),
            AccountMeta::new_readonly(stake_authority.0, false),
            AccountMeta::new(token_account_stake_pool, false),
            AccountMeta::new_readonly(*mint, false),
        ],
    ))
}

/// Creates [Instruction::Stake] instruction which transfer `amount` from `token_account_source` to `token_account_stake_target`.
/// If `stake_account` initialized, resets timer.
/// Locks amount for period of time defined in `stake_pool`.
///
/// Accounts:
///  * `system_program`                - *program, implicit*
///  * `sysvar_rent`                   - *program, implicit* to create `stake_account` which will be rent except if needed
///  * `clock`                         - *program, implicit*
///  * `spl_token`                     - *program, implicit*
///  * `fee_payer`                     - *signer, payer* pays for account creation
///  * `stake_pool`                    - account of stake pool used
///  * `stake_authority`               - *derived*  as in [Instruction::InitializeStakePool]
///  * `token_account_source`          - *mutable* represents user and has approval for input amount
///  * `token_account_stake_target`    - *derived, mutable, implicit*
///  * `viewer_stake_account`          - *implicit, derived, mutable* from `user_wallet` and `stake_authority`
///  * `user_wallet`                   - *signer* something to which user has private key of
///  * `stake_pool_owner`              - *optional, signer* owner of stake pool can create stake for viewer, in this case `user_wallet` may not be signer  
///
/// Notes:
/// - current design does not creates token account to stake tokens, just counts amount in user stake.
/// - stake instruction is same instruction as initialize stake, so it could be made different by having separate stake (it will reduce amount of accounts during stake invocation)
#[allow(clippy::too_many_arguments)]
pub fn stake(
    fee_payer: &SignerPubkey,
    stake_pool: &Pubkey,
    token_account_source: &TokenAccountPubkey,
    user_wallet: &PossiblySignerPubkey,
    stake_pool_owner: &Option<SignerPubkey>,
    input: StakeInput,
) -> Result<(solana_program::instruction::Instruction, Pubkey), ProgramError> {
    let (stake_authority, _) =
        Pubkey::find_program_address_for_pubkey(stake_pool, &stake_viewer_program_id());
    let token_account_stake_target = Pubkey::create_with_seed(
        &stake_authority,
        "ViewerStakePool::token_account",
        &spl_token::id(),
    )?;

    let viewer_stake_account = Pubkey::create_with_seed_for_pubkey(
        &stake_authority,
        user_wallet,
        &stake_viewer_program_id(),
    )?;
    let mut accounts = vec![
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
        AccountMeta::new_readonly(sysvar::clock::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(*fee_payer, true),
        AccountMeta::new_readonly(*stake_pool, false),
        AccountMeta::new_readonly(stake_authority, false),
        AccountMeta::new(*token_account_source, false),
        AccountMeta::new(token_account_stake_target, false),
        AccountMeta::new(viewer_stake_account.0, false),
    ];

    if stake_pool_owner.is_none() {
        accounts.push(AccountMeta::new_readonly(*user_wallet, true));
    }
    else {
        accounts.push(AccountMeta::new_readonly(*user_wallet, false));
        accounts.push(AccountMeta::new_readonly(stake_pool_owner.unwrap(), true));
    }

    Ok((
        solana_program::instruction::Instruction::new_with_borsh(
            crate::id(),
            &Instruction::Stake(input),
            accounts,
        ),
        viewer_stake_account.0,
    ))
}

/// Creates [Instruction::Unstake] instruction which transfer `amount` from `token_account_stake_source` to `token_account_target` if and only if now is more than [crate::state::ViewerStake::staked_until]
/// Resets unlock.
///
/// If `token_account_target` is associated token account of `user_wallet` 
///
/// Accounts:
///  * `sysvar_clock`               - *program, implicit* used to check lock maturity
///  * `spl_token`                  - *program, implicit*
///  * `fee_payer`                  - *mutable, payer*, in case of full unlock, will transfers sols from lock onto this account
///  * `stake_pool`                 - state account initialized
///  * `stake_authority`            - *implicit*, derived from `owner`
///  * `token_account_target`       - *mutable* represent user account for SAO tokens
///  * `token_account_stake_source` - *derived, mutable, implicit*
///  * `user_stake_account`         - *implicit, derived, mutable* from `user_wallet` and `stake_authority`
///  * `user_wallet`                - *signer* owner of `user_stake_account`, can transfer to any `token_account_stake_source` if signed. 
///  * `stake_pool_owner`           - *optional, signer*, if `user_wallet` is not signer,  than only to if `token_account_target` associated token account signature of `stake_pool_owner` is enough
pub fn unstake(
    stake_pool: &Pubkey,
    token_account_target: &TokenAccountPubkey,
    user_wallet: &PossiblySignerPubkey,
    fee_payer: &Pubkey,
    stake_pool_owner: &Option<SignerPubkey>,
) -> Result<solana_program::instruction::Instruction, ProgramError> {
    let (stake_authority, _) =
        Pubkey::find_program_address_for_pubkey(stake_pool, &stake_viewer_program_id());
    let token_account_stake_source = Pubkey::create_with_seed(
        &stake_authority,
        "ViewerStakePool::token_account",
        &spl_token::id(),
    )?;
    let user_stake_account = Pubkey::create_with_seed_for_pubkey(
        &stake_authority,
        user_wallet,
        &stake_viewer_program_id(),
    )?;
    let mut accounts = vec![
        AccountMeta::new_readonly(sysvar::clock::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new(*fee_payer, false),
        AccountMeta::new_readonly(*stake_pool, false),
        AccountMeta::new_readonly(stake_authority, false),
        AccountMeta::new(*token_account_target, false),
        AccountMeta::new(token_account_stake_source, false),
        AccountMeta::new(user_stake_account.0, false),
    ];

    if stake_pool_owner.is_none() {
        accounts.push(AccountMeta::new_readonly(*user_wallet, true));
    }
    else {
        accounts.push(AccountMeta::new_readonly(*user_wallet, false));
        accounts.push(AccountMeta::new_readonly(stake_pool_owner.unwrap(), true));
    }

    Ok(solana_program::instruction::Instruction::new_with_borsh(
        crate::id(),
        &Instruction::Unstake,
        accounts,
    ))
}

#[cfg(test)]
mod tests {
    use crate::{types::Rank};

    use super::{InitializeStakePoolInput, Instruction};
    use borsh::*;

    #[test]
    fn test() {
        let input = Instruction::InitializeStakePool(InitializeStakePoolInput {
            ranks: [
                Rank {
                    minimal_staking_time: 0,
                    amount: 100,
                },
                Rank {
                    minimal_staking_time: 30 * 60,
                    amount: 200,
                },
                Rank {
                    minimal_staking_time: 60 * 60,
                    amount: 300,
                },
                Rank {
                    minimal_staking_time: 2 * 60 * 60,
                    amount: 500,
                },
            ],
        });

        let data = hex::encode(input.try_to_vec().unwrap());
        assert_eq!(data, "00000000000000000064000000000000000807000000000000c800000000000000100e0000000000002c01000000000000201c000000000000f401000000000000");

        #[derive(Debug, BorshDeserialize, BorshSerialize, BorshSchema)]
        pub struct InitializeStakePoolInput2 {
            pub ranks: Vec<Rank>,
        }

        #[derive(Debug, BorshDeserialize, BorshSerialize, BorshSchema)]
        pub enum Instruction2 {
            InitializeStakePool(InitializeStakePoolInput2),
        }

        let input = Instruction2::InitializeStakePool(InitializeStakePoolInput2 {
            ranks: vec![
                Rank {
                    minimal_staking_time: 0,
                    amount: 100,
                },
                Rank {
                    minimal_staking_time: 30 * 60,
                    amount: 200,
                },
                Rank {
                    minimal_staking_time: 60 * 60,
                    amount: 300,
                },
                Rank {
                    minimal_staking_time: 2 * 60 * 60,
                    amount: 500,
                },
            ],
        });

        let data = hex::encode(input.try_to_vec().unwrap());
        assert_eq!(data, "0004000000000000000000000064000000000000000807000000000000c800000000000000100e0000000000002c01000000000000201c000000000000f401000000000000");

        let input = Instruction::Unstake;

        let data = hex::encode(input.try_to_vec().unwrap());
        assert_eq!(data, "02");
    }
}
