use sator_sdk::{borsh::*, ensure};
use sator_sdk::invoke::{self, ProgramPubkeySignature};
use sator_sdk::state::StateVersion;
use sator_sdk::types::*;
use sator_sdk::{ensure_derived, ensure_eq, ensure_owner, program::*};
use solana_program::clock::Clock;
use solana_program::msg;
use solana_program::program_error::ProgramError;
use solana_program::program_pack::Pack;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

use crate::errors;
use crate::instruction::Instruction;
use crate::state::{ViewerStake, ViewerStakePool};

// Program entrypoint's implementation
#[allow(dead_code)]
pub fn process_instruction(
    program_id: &ProgramPubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = Instruction::deserialize_const(instruction_data)?;
    match instruction {
        Instruction::InitializeStakePool(input) => {
            msg!("Instruction::InitializeStakePool");
            match accounts {
                [system_program, sysvar_rent, spl_token, fee_payer, stake_pool_owner, stake_pool, stake_authority, token_account, mint, ..] => {
                    initialize_stake_pool(
                        program_id,
                        system_program,
                        sysvar_rent,
                        spl_token,
                        fee_payer,
                        stake_pool_owner,
                        stake_pool,
                        stake_authority,
                        token_account,
                        mint,
                        &input,
                    )
                }
                _ => Err(ProgramError::NotEnoughAccountKeys),
            }
        }
        Instruction::Stake(input) => {
            msg!("Instruction::Stake");
            match accounts {
                [
                    system_program,
                    sysvar_rent,
                    clock,
                    spl_token,
                    fee_payer,
                    stake_pool,
                    stake_authority,
                    token_account_source,
                    token_account_stake_target,
                    viewer_stake_account,
                    user_wallet,
                    ..
                ] => {
                    let stake_pool_owner = accounts.get(11);
                    stake(
                        program_id,
                        system_program,
                        sysvar_rent,
                        clock,
                        spl_token,
                        fee_payer,
                        stake_pool,
                        stake_authority,
                        token_account_source,
                        token_account_stake_target,
                        viewer_stake_account,
                        user_wallet,
                        stake_pool_owner,                    
                        input,
                    )
                }
                _ => Err(ProgramError::NotEnoughAccountKeys),
            }
        }

        Instruction::Unstake => {
            msg!("Instruction::Unstake");
            match accounts {
                [
                    sysvar_clock,
                    spl_token,
                    fee_payer,
                    stake_pool,
                    stake_authority,
                    token_account_target,
                    token_account_stake_source,
                    user_stake_account,
                    user_wallet,
                    ..
                ] => {
                    let stake_pool_owner = accounts.get(9);
                    unstake(
                        program_id,
                        sysvar_clock,
spl_token,
fee_payer,
stake_pool,
stake_authority,
token_account_target,
token_account_stake_source,
user_stake_account,
user_wallet,
stake_pool_owner,
                    )
                }
                _ => Err(ProgramError::NotEnoughAccountKeys),
            }
        }
    }
}

fn initialize_stake_pool<'a>(
    program_id: &ProgramPubkey,
    system_program: &AccountInfo<'a>,
    rent: &AccountInfo<'a>,
    spl_token: &AccountInfo<'a>,
    fee_payer: &AccountInfo<'a>,
    stake_pool_owner: &AccountInfo<'a>,
    stake_pool: &AccountInfo<'a>,
    stake_authority: &AccountInfo<'a>,
    token_account: &AccountInfo<'a>,
    mint: &AccountInfo<'a>,
    input: &crate::instruction::InitializeStakePoolInput,
) -> ProgramResult {
    stake_pool_owner.is_signer()?;
    stake_pool.is_signer()?;
    let (stake_authority_pubkey, bump_seed, token_account_pubkey) =
        derive_token_account(stake_pool, program_id)?;

    is_derived(stake_authority_pubkey, stake_authority)?;
    is_derived(token_account_pubkey, token_account)?;

    let rent_state = Rent::from_account_info(rent)?;
    let lamports = rent_state.minimum_balance(ViewerStakePool::LEN);
    invoke::create_account(
        fee_payer.clone(),
        stake_pool.clone(),
        lamports,
        ViewerStakePool::LEN as u64,
        program_id,
        system_program,
        spl_token,
    )?;

    let authority_signature = ProgramPubkeySignature::new(stake_pool, bump_seed);

    let lamports = rent_state.minimum_balance(spl_token::state::Account::LEN);
    invoke::create_account_with_seed_signed(
        system_program,
        &fee_payer,
        &token_account,
        stake_authority,
        "ViewerStakePool::token_account",
        lamports,
        spl_token::state::Account::LEN as u64,
        &spl_token::id(),
        &authority_signature,
    )?;

    invoke::initialize_token_account_signed(
        token_account,
        &mint,
        stake_authority,
        rent,
        &authority_signature,
    )?;

    let mut state = stake_pool.deserialize::<ViewerStakePool>()?;
    state.ranks = input.ranks.clone();
    state.owner = stake_pool_owner.pubkey();
    state.version = StateVersion::V1;
    state.serialize_const(&mut *stake_pool.try_borrow_mut_data()?)?;

    Ok(())
}

fn derive_token_account(
    stake: &AccountInfo,
    program_id: &ProgramPubkey,
) -> Result<(Pubkey, u8, Pubkey), ProgramError> {
    let (stake_authority_pubkey, bump_seed) = {
        let (stake_authority_pubkey, bump_seed) =
            Pubkey::find_program_address_for_pubkey(&stake.pubkey(), program_id);
        (stake_authority_pubkey, bump_seed)
    };
    let derived = Pubkey::create_with_seed(
        &stake_authority_pubkey,
        "ViewerStakePool::token_account",
        &spl_token::id(),
    )?;
    Ok((stake_authority_pubkey, bump_seed, derived))
}

pub fn stake<'a>(
    program_id: &ProgramPubkey,
    system_program: &AccountInfo<'a>,
    sysvar_rent: &AccountInfo<'a>,
    clock: &AccountInfo<'a>,
    spl_token: &AccountInfo<'a>,
    fee_payer: &AccountInfo<'a>,
    stake_pool: &AccountInfo<'a>,
    stake_authority: &AccountInfo<'a>,
    token_account_source: &AccountInfo<'a>,
    token_account_stake_target: &AccountInfo<'a>,
    viewer_stake_account: &AccountInfo<'a>,
    user_wallet: &AccountInfo<'a>,
    stake_pool_owner: Option<&AccountInfo<'a>>,
    input: crate::instruction::StakeInput,
) -> ProgramResult {
    ensure_eq!(
        program_id,
        stake_pool.owner,
        errors::Error::StakePoolMustBeOwnedByThisContract
    );
    let stake_pool_state = stake_pool.deserialize::<ViewerStakePool>()?;
    stake_pool_state.initialized()?;
    if let Some(stake_pool_owner) = stake_pool_owner {
        ensure_owner!(
            stake_pool_owner,
            stake_pool_state,
            errors::Error::StakePoolOwnerMustOwnStake
        );    
        stake_pool_owner.is_signer()?;
    } else {
        user_wallet.is_signer()?;
    }

    let can_restake = user_wallet.is_signer().is_ok();
    ensure!(
        can_restake || stake_pool_owner.map_or(false, |x| x.is_signer().is_ok()),
        errors::Error::StakeForViewerMustBeSignedByUserWalletOrPoolAdmin
    );
    
    let clock = Clock::from_account_info(clock)?;
    if input.duration < stake_pool_state.ranks[0].minimal_staking_time {
        return errors::Error::StakeStakingTimeMustBeMoreThanMinimal.into();
    }

    let (stake_authority_pubkey, bump_seed, token_account_pubkey) =
        derive_token_account(stake_pool, program_id)?;

    let (viewer_stake_account_pubkey, seed) = Pubkey::create_with_seed_for_pubkey(
        &stake_authority_pubkey,
        &user_wallet.pubkey(),
        program_id,
    )?;

    ensure_derived!(
        stake_authority_pubkey,
        stake_authority,
        errors::Error::StakeAuthorityMustBeDerivedFromStake
    );
    ensure_derived!(
        token_account_pubkey,
        token_account_stake_target,
        errors::Error::StakeTokenAccountMustBeDerivedFromStake
    );
    ensure_derived!(
        viewer_stake_account_pubkey,
        viewer_stake_account,
        errors::Error::StakeUserMustBeDerivedFromUserToken
    );

    let authority_signature = ProgramPubkeySignature::new(stake_pool, bump_seed);
    let stake_user_account_state = if viewer_stake_account.data_is_empty() {        
        // new stake
        let stake_user_account_state = ViewerStake {
            amount: input.amount,
            owner: user_wallet.pubkey(),
            staked_until: clock.unix_timestamp + input.duration,
            version: StateVersion::V1,
            staked_at: clock.unix_timestamp,
        };
        let rent_state = Rent::from_account_info(sysvar_rent)?;
        let lamports = rent_state.minimum_balance(ViewerStake::LEN);
        invoke::create_account_with_seed_signed(
            system_program,
            &fee_payer,
            &viewer_stake_account,
            stake_authority,
            &seed[..],
            lamports,
            ViewerStake::LEN as u64,
            program_id,
            &authority_signature,
        )?;
        stake_user_account_state
    } else {
        ensure_eq!(
            program_id,
            viewer_stake_account.owner,
            errors::Error::StakeUserAccountMustBeOwnedByThisContract
        );
        let mut stake_user_account_state = viewer_stake_account.deserialize::<ViewerStake>()?;
        stake_user_account_state.initialized()?;

        if input.duration < stake_user_account_state.duration() {
            return errors::Error::StakeStakingTimeMustBeMoreThanPrevious.into();
        }

        ensure_owner!(
            user_wallet,
            stake_user_account_state,
            errors::Error::UserWalletMustBeOwnerOfViewerStakeAccount
        );
        stake_user_account_state.staked_until = clock.unix_timestamp + input.duration;
        // existing stake just adds on top
        stake_user_account_state.amount += input.amount;
        stake_user_account_state.staked_at = clock.unix_timestamp;
        stake_user_account_state
    };

    // transfer amount from provided user token account into stake pool
    let signer = if user_wallet.is_signer {
        user_wallet
    }
    else {
        stake_pool_owner.unwrap()
    };

    invoke::spl_token_transfer(
        spl_token,
        token_account_source,
        token_account_stake_target,
        signer,
        input.amount,
    )?;
    stake_user_account_state.serialize_const(&mut *viewer_stake_account.try_borrow_mut_data()?)?;
    Ok(())
}

fn unstake<'a>(
    program_id: &ProgramPubkey,
    sysvar_clock: &AccountInfo<'a>,
    spl_token: &AccountInfo<'a>,
    fee_payer: &AccountInfo<'a>,
    stake_pool: &AccountInfo<'a>,
    stake_authority: &AccountInfo<'a>,
    token_account_target: &AccountInfo<'a>,
    token_account_stake_source: &AccountInfo<'a>,
    user_stake_account: &AccountInfo<'a>,
    user_wallet: &AccountInfo<'a>,
    stake_pool_owner: Option<&AccountInfo<'a>>,    
) -> ProgramResult {
    let viewer_stake_pool_state = stake_pool.deserialize::<ViewerStakePool>()?;
    viewer_stake_pool_state.initialized()?;

    if let Some(stake_pool_owner) = stake_pool_owner {
        stake_pool_owner.is_signer()?;
        ensure_owner!(
            stake_pool_owner,
            viewer_stake_pool_state,
            errors::Error::StakePoolOwnerMustOwnStake
        );
        let borrow = token_account_target.try_borrow_data().unwrap();
        let token_account_data = spl_token::state::Account::unpack(&borrow)?;
        let associated_token_address = spl_associated_token_account::get_associated_token_address(&user_wallet.pubkey(), &token_account_data.mint);
        if associated_token_address != token_account_target.pubkey() {
            return errors::Error::AdminCanUnstakeOnlyToUserWalletAssosiatedTokenAddress.into();
        }
    }
    else {
        user_wallet.is_signer()?;
    }

    ensure_eq!(
        program_id,
        stake_pool.owner,
        errors::Error::StakePoolMustBeOwnedByThisContract
    );
    ensure_eq!(
        program_id,
        user_stake_account.owner,
        errors::Error::StakeUserAccountMustBeOwnedByThisContract
    );    

    let user_stake_account_state = user_stake_account.deserialize::<ViewerStake>()?;
    user_stake_account_state.initialized()?;

    ensure_derived!(
        user_stake_account_state.owner, 
        user_wallet,
        errors::Error::UserWalletMustBeOwnerOfViewerStakeAccount
    );

    let clock = Clock::from_account_info(sysvar_clock)?;
    if user_stake_account_state.staked_until > clock.unix_timestamp {
        return errors::Error::UnstakeCanBeDoneOnlyAfterStakeTimeLapsed.into();
    }

    let (stake_authority_pubkey, bump_seed, token_account_stake_source_pubkey) =
        derive_token_account(stake_pool, program_id)?;


    ensure_derived!(
        token_account_stake_source_pubkey, 
        token_account_stake_source,
        errors::Error::StakePoolTokenAccountMustBeDerivedFromPool
    );

    let (stake_account_pubkey, _) = Pubkey::create_with_seed_for_pubkey(
        &stake_authority_pubkey,
        &user_wallet.pubkey(),
        program_id,
    )?;

    is_derived(stake_account_pubkey, user_stake_account)?;

    let authority_signature = ProgramPubkeySignature::new(stake_pool, bump_seed);

    // transfer previously staked amount to user token account
    invoke::spl_token_transfer_signed(
        spl_token,
        token_account_stake_source,
        token_account_target,
        stake_authority,
        user_stake_account_state.amount,
        &authority_signature,
    )?;

    burn_account(user_stake_account, fee_payer);

    Ok(())
}
