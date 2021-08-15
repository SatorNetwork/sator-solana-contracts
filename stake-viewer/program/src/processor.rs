use std::borrow::Borrow;
use std::error::Error;

use sator_sdk::invoke::{self, ProgramPubkeySignature};
use sator_sdk::program::*;
use sator_sdk::state::StateVersion;
use sator_sdk::types::*;
use sator_sdk::{borsh::*, is_owner};
use solana_program::clock::Clock;
use solana_program::msg;
use solana_program::program_error::{PrintProgramError, ProgramError};
use solana_program::program_pack::Pack;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, pubkey::Pubkey,
};

use crate::errors;
use crate::instruction::Instruction;
use crate::state::{ViewerStake, ViewerStakePool};
use borsh::BorshSerialize;

// Program entrypoint's implementation
pub fn process_instruction(
    program_id: &ProgramPubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = Instruction::deserialize_const(instruction_data)?;
    match instruction {
        Instruction::InitializeStakePool(input) => {
            msg!("Instruction::InitializeStake");
            match accounts {
                [system_program, sysvar_rent, spl_token, fee_payer, stake_pool_owner, stake_pool, stake_authority, token_account, mint, ..] => {
                    initialize_stake(
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
                [system_program, rent, clock, spl_token, fee_payer, stake_pool, stake_pool_owner, stake_authority, token_account_user, token_account_stake_target, user_stake_account, ..] => {
                    stake(
                        program_id,
                        system_program,
                        rent,
                        clock,
                        spl_token,
                        fee_payer,
                        stake_pool,
                        stake_pool_owner,
                        stake_authority,
                        token_account_user,
                        token_account_stake_target,
                        user_stake_account,
                        input,
                    )
                }
                _ => Err(ProgramError::NotEnoughAccountKeys),
            }
        }

        Instruction::Unstake => {
            msg!("Instruction::Unstake");
            match accounts {
                [clock, spl_token, stake_pool, stake_authority, token_account_user, token_account_stake_source, stake_account, stake_pool_owner, ..] => {
                    unstake(
                        program_id,
                        clock,
                        spl_token,
                        stake_pool,
                        stake_authority,
                        token_account_user,
                        token_account_stake_source,
                        stake_account,
                        stake_pool_owner,
                    )
                }
                _ => Err(ProgramError::NotEnoughAccountKeys),
            }
        }
    }
}

fn initialize_stake<'a>(
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
    )?;

    let authority_signature = ProgramPubkeySignature::new(stake_pool, bump_seed);

    let lamports = rent_state.minimum_balance(spl_token::state::Account::LEN);
    invoke::create_account_with_seed_signed(
        system_program,
        &stake_pool_owner,
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

fn stake<'a>(
    program_id: &ProgramPubkey,
    system_program: &AccountInfo<'a>,
    rent: &AccountInfo<'a>,
    clock: &AccountInfo<'a>,
    spl_token: &AccountInfo<'a>,
    fee_payer: &AccountInfo<'a>,
    stake_pool: &AccountInfo<'a>,
    stake_pool_owner: &AccountInfo<'a>,
    stake_authority: &AccountInfo<'a>,
    token_account_source: &AccountInfo<'a>,
    token_account_stake_target: &AccountInfo<'a>,
    stake_account: &AccountInfo<'a>,
    input: crate::instruction::StakeInput,
) -> ProgramResult {
    stake_pool.is_owner(program_id)?;
    let stake_pool_state = stake_pool.deserialize::<ViewerStakePool>()?;
    stake_pool_state.initialized()?;
    is_owner!(stake_pool_owner, stake_pool_state);
    let clock = Clock::from_account_info(clock)?;
    if input.duration < stake_pool_state.ranks[0].minimal_staking_time {
        return errors::Error::StakeStakingTimeMustBeMoreThanMinimal.into();
    }

    let (stake_authority_pubkey, bump_seed, token_account_pubkey) =
        derive_token_account(stake_pool, program_id)?;

    let (stake_account_pubkey, seed) = Pubkey::create_with_seed_for_pubkey(
        &stake_authority_pubkey,
        &token_account_source.pubkey(),
        program_id,
    )?;

    is_derived(stake_authority_pubkey, stake_authority)?;
    is_derived(token_account_pubkey, token_account_stake_target)?;
    is_derived(stake_account_pubkey, stake_account)?;

    let authority_signature = ProgramPubkeySignature::new(stake_pool, bump_seed);
    let stake_account_state = if stake_account.data_is_empty() {
        let stake_account_state = ViewerStake {
            amount: input.amount,
            owner: token_account_source.pubkey(),
            staked_until: clock.unix_timestamp + input.duration,
            version: StateVersion::V1,
            staked_at: clock.unix_timestamp,
        };
        let rent_state = Rent::from_account_info(rent)?;
        let lamports = rent_state.minimum_balance(ViewerStake::LEN);
        invoke::create_account_with_seed_signed(
            system_program,
            &fee_payer,
            &stake_account,
            stake_authority,
            &seed[..],
            lamports,
            ViewerStake::LEN as u64,
            program_id,
            &authority_signature,
        )?;
        stake_account_state
    } else {
        stake_account.is_owner(program_id)?;
        let mut stake_account_state = stake_account.deserialize::<ViewerStake>()?;
        stake_account_state.initialized()?;

        if input.duration < stake_account_state.duration() {
            return errors::Error::StakeStakingTimeMustBeMoreThanPrevious.into();
        }

        is_owner!(token_account_source, stake_account_state);
        stake_account_state.staked_until = clock.unix_timestamp + input.duration;
        stake_account_state.amount += input.amount;
        stake_account_state.staked_at = clock.unix_timestamp;
        stake_account_state
    };

    invoke::spl_token_transfer(
        spl_token,
        token_account_source,
        token_account_stake_target,
        stake_pool_owner,
        input.amount,
    )?;
    stake_account_state.serialize_const(&mut *stake_account.try_borrow_mut_data()?)?;
    Ok(())
}

fn unstake<'a>(
    program_id: &ProgramPubkey,
    clock: &AccountInfo<'a>,
    spl_token: &AccountInfo<'a>,
    stake_pool: &AccountInfo<'a>,
    stake_authority: &AccountInfo<'a>,
    token_account_user: &AccountInfo<'a>,
    token_account_stake_source: &AccountInfo<'a>,
    user_stake_account: &AccountInfo<'a>,
    stake_pool_owner: &AccountInfo<'a>,
) -> ProgramResult {
    stake_pool_owner.is_signer()?;

    let viewer_stake_pool_state = stake_pool.deserialize::<ViewerStakePool>()?;
    viewer_stake_pool_state.initialized()?;
    stake_pool.is_owner(program_id)?;
    is_owner!(stake_pool_owner, viewer_stake_pool_state);

    let user_stake_account_state = user_stake_account.deserialize::<ViewerStake>()?;
    user_stake_account_state.initialized()?;
    user_stake_account.is_owner(program_id)?;
    is_derived(user_stake_account_state.owner, token_account_user)?;

    let clock = Clock::from_account_info(clock)?;
    if user_stake_account_state.staked_until > clock.unix_timestamp {
        return errors::Error::UnstakeCanBeDoneOnlyAfterStakeTimeLapsed.into();
    }

    let (stake_authority_pubkey, bump_seed, token_account_stake_pubkey) =
        derive_token_account(stake_pool, program_id)?;
    is_derived(token_account_stake_pubkey, token_account_stake_source)?;

    let (stake_account_pubkey, _) = Pubkey::create_with_seed_for_pubkey(
        &stake_authority_pubkey,
        &token_account_user.pubkey(),
        program_id,
    )?;

    is_derived(stake_account_pubkey, user_stake_account)?;

    let authority_signature = ProgramPubkeySignature::new(stake_pool, bump_seed);

    invoke::spl_token_transfer_signed(
        spl_token,
        token_account_stake_source,
        token_account_user,
        stake_authority,
        user_stake_account_state.amount,
        &authority_signature,
    )?;

    burn_account(user_stake_account, stake_pool_owner);

    Ok(())
}
