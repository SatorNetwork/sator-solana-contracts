use std::borrow::Borrow;
use std::error::Error;

use solana_program::clock::Clock;
use solana_program::msg;
use solana_program::program_error::{PrintProgramError, ProgramError};
use solana_program::program_pack::Pack;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, pubkey::Pubkey,
};

use crate::instruction::Instruction;
use crate::{program_id, errors};
use crate::sdk::invoke::ProgramPubkeySignature;
use crate::sdk::program::{AccountPatterns, PubkeyPatterns, burn_account, is_derived};
use crate::sdk::types::{ProgramPubkey, SignerPubkey};
use crate::sdk::{
    borsh::{BorshDeserializeConst, BorshSerializeConst},
    invoke,
};
use crate::state::{StateVersion, ViewerStake, ViewerStakePool};
use borsh::{BorshDeserialize, BorshSerialize};

// Program entrypoint's implementation
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("sator_stake_viewer::process_instruction");
    let instruction = Instruction::deserialize_const(instruction_data)?;
    match instruction {
        Instruction::InitializeStakePool(input) => {
            msg!("Instruction::InitializeStake");
            match accounts {
                [system_program, rent, spl_token, owner, stake_pool, stake_authority, token_account, mint, ..] => {
                    initialize_stake(
                        program_id,
                        system_program,
                        rent,
                        spl_token,
                        owner,
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
                [system_program, rent, clock, spl_token, wallet, stake_pool, stake_authority, token_account_source, token_account_stake_target, stake_account, ..] => {
                    stake(
                        system_program,
                        rent,
                        clock,
                        spl_token,
                        wallet,
                        stake_pool,
                        stake_authority,
                        token_account_source,
                        token_account_stake_target,
                        stake_account,
                        input,
                    )
                }
                _ => Err(ProgramError::NotEnoughAccountKeys),
            }
        }

        Instruction::Unstake => {
            msg!("Instruction::Unstake");
            match accounts {
                [clock, spl_token, wallet, stake_pool, stake_authority, token_account_target, token_account_stake_source, stake_account, ..] => {
                    unstake(
                        clock,
                        spl_token,
                        wallet,
                        stake_pool,
                        stake_authority,
                        token_account_target,
                        token_account_stake_source,
                        stake_account,
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
    owner: &AccountInfo<'a>,
    stake_pool: &AccountInfo<'a>,
    stake_authority: &AccountInfo<'a>,
    token_account: &AccountInfo<'a>,
    mint: &AccountInfo<'a>,
    input: &crate::instruction::InitializeStakePoolInput,
) -> ProgramResult {
    owner.is_signer()?;
    stake_pool.is_signer()?;
    let (stake_authority_pubkey, bump_seed, token_account_pubkey) = derive_token_account(stake_pool)?;

    is_derived(stake_authority_pubkey, stake_authority)?;
    is_derived(token_account_pubkey, token_account)?;

    let rent_state = Rent::from_account_info(rent)?;
    let lamports = rent_state.minimum_balance(ViewerStakePool::LEN);
    invoke::create_account(
        owner.clone(),
        stake_pool.clone(),
        lamports,
        ViewerStakePool::LEN as u64,
        program_id,
        system_program,
    )?;
    let lamports = rent_state.minimum_balance(spl_token::state::Account::LEN);

    let authority_signature = ProgramPubkeySignature::new(stake_pool, bump_seed);

    invoke::create_account_with_seed_signed(
        system_program,
        &owner,
        &token_account,
        stake_authority,
        "ViewerStakePool::token_account".to_string(),
        lamports,
        spl_token::state::Account::LEN as u64,
        &spl_token::id(),
        bump_seed,
        &authority_signature,
    )?;

    invoke::initialize_token_account_signed(
        token_account,
        &mint,
        stake_authority,
        rent,
        bump_seed,
        &authority_signature,
    )?;

    let mut state = stake_pool.deserialize::<ViewerStakePool>()?;
    state.ranks = input.ranks.clone();
    state.owner = owner.pubkey();
    state.serialize_const(&mut *stake_pool.try_borrow_mut_data()?)?;

    Ok(())
}

fn derive_token_account(stake: &AccountInfo) -> Result<(Pubkey, u8, Pubkey), ProgramError> {
    let (stake_authority_pubkey, bump_seed) = derive_stake_authority_account(stake);
    let derived = Pubkey::create_with_seed(
        &stake_authority_pubkey,
        "ViewerStakePool::token_account",
        &spl_token::id(),
    )?;
    Ok((stake_authority_pubkey, bump_seed, derived))
}

fn derive_stake_authority_account(stake: &AccountInfo) -> (Pubkey, u8) {
    let (stake_authority_pubkey, bump_seed) =
        Pubkey::find_program_address_for_pubkey(&stake.pubkey(), &crate::program_id());
    (stake_authority_pubkey, bump_seed)
}

fn stake<'a>(
    system_program: &AccountInfo<'a>,
    rent: &AccountInfo<'a>,
    clock: &AccountInfo<'a>,
    spl_token: &AccountInfo<'a>,
    wallet: &AccountInfo<'a>,
    stake_pool: &AccountInfo<'a>,
    stake_authority: &AccountInfo<'a>,
    token_account_source: &AccountInfo<'a>,
    token_account_stake_target: &AccountInfo<'a>,
    stake_account: &AccountInfo<'a>,
    input: crate::instruction::StakeInput,
) -> ProgramResult {
    let stake_pool_state = stake_pool.deserialize::<ViewerStakePool>()?;
    let clock = Clock::from_account_info(clock)?;
    if input.duration < stake_pool_state.ranks[0].minimal_staking_time {
        return errors::Error::StakeStakingTimeMustBeMoreThanMinimal.into();
    }
    let (stake_authority_pubkey, bump_seed, token_account_pubkey) = derive_token_account(stake_pool)?;

    let (stake_account_pubkey, seed) = Pubkey::create_with_seed_for_pubkey(
        &stake_authority_pubkey,
        &wallet.pubkey(),
        &program_id(),
    )?;

    is_derived(stake_authority_pubkey, stake_authority)?;
    is_derived(token_account_pubkey, token_account_stake_target)?;
    is_derived(stake_account_pubkey, stake_account)?;

    let authority_signature = ProgramPubkeySignature::new(stake_pool, bump_seed);
    let stake_account_state = if stake_account.data_is_empty() {        
        let stake_account_state = ViewerStake {
            amount: input.amount,
            owner: wallet.pubkey(),
            staked_until: clock.unix_timestamp + input.duration,
            version: StateVersion::V1,
            staked_at: clock.unix_timestamp,
        };
        let rent_state = Rent::from_account_info(rent)?;
        let lamports = rent_state.minimum_balance(ViewerStake::LEN);
        invoke::create_account_with_seed_signed(
            system_program,
            &wallet,
            &stake_account,
            stake_authority,
            seed,
            lamports,
            ViewerStake::LEN as u64,
            &program_id(),
            bump_seed,
            &authority_signature,
        )?;
        stake_account_state
    } else {
        let mut stake_account_state = stake_account.deserialize::<ViewerStake>()?;
        is_derived(stake_account_state.owner, wallet)?;
        stake_account_state.staked_until = clock.unix_timestamp + input.duration;
        stake_account_state.amount += input.amount;
        stake_account_state.staked_at = clock.unix_timestamp;
        stake_account_state
    };

    invoke::spl_token_transfer(
        spl_token,
        token_account_source,
        token_account_stake_target,
        wallet,
        input.amount,
    )?;
    stake_account_state.serialize_const(&mut *stake_account.try_borrow_mut_data()?)?;
    Ok(())
}

fn unstake<'a>(
    clock: &AccountInfo<'a>,
    spl_token: &AccountInfo<'a>,
    wallet: &AccountInfo<'a>,
    stake_pool: &AccountInfo<'a>,
    stake_authority: &AccountInfo<'a>,
    token_account_target: &AccountInfo<'a>,
    token_account_stake_source: &AccountInfo<'a>,
    stake_account: &AccountInfo<'a>,
) -> ProgramResult {    
    let stake_account_state = stake_account.deserialize::<ViewerStake>()?;
    is_derived(stake_account_state.owner, wallet)?;
    wallet.is_signer()?;
    let clock = Clock::from_account_info(clock)?;
    if stake_account_state.staked_until > clock.unix_timestamp {
        return errors::Error::UnstakeCanBeDoneOnlyAfterStakeTimeLapsed.into();
    }

    let (stake_authority_pubkey, bump_seed, token_account_stake_pubkey) = derive_token_account(stake_pool)?;
    is_derived(token_account_stake_pubkey, token_account_stake_source)?;

    let (stake_account_pubkey, _) = Pubkey::create_with_seed_for_pubkey(
        &stake_authority_pubkey,
        &wallet.pubkey(),
        &program_id(),
    )?;

    is_derived(stake_account_pubkey, stake_account)?;

    let authority_signature = ProgramPubkeySignature::new(stake_pool, bump_seed);
    
    let stake_pool_state = stake_pool.deserialize::<ViewerStakePool>()?;
    
    let reward_amount = stake_pool_state.calculate_reward(stake_account_state)?;
    msg!("Unstake::reward_amount {:?}", reward_amount);
    invoke::spl_token_transfer_signed(
        spl_token,
        token_account_stake_source,
        token_account_target,
        stake_authority,
        reward_amount,
        &authority_signature,
    )?;
    
    burn_account(stake_account, wallet);

    Ok(())
}
