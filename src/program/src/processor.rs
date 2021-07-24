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
use crate::program_id;
use crate::sdk::invoke::ProgramPubkeySignature;
use crate::sdk::program::{is_derived, AccountPatterns, PubkeyPatterns};
use crate::sdk::types::{ProgramPubkey, SignerPubkey};
use crate::sdk::{
    borsh::{BorshDeserializeConst, BorshSerializeConst},
    invoke,
};
use crate::state::{StateVersion, ViewerLock, ViewerStake};
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
        Instruction::InitializeStake(input) => {
            msg!("Instruction::InitializeStake");
            match accounts {
                [system_program, rent, spl_token, owner, stake, stake_authority, token_account, mint, ..] => {
                    initialize_stake(
                        program_id,
                        system_program,
                        rent,
                        spl_token,
                        owner,
                        stake,
                        stake_authority,
                        token_account,
                        mint,
                        &input,
                    )
                }
                _ => Err(ProgramError::NotEnoughAccountKeys),
            }
        }
        Instruction::Lock(input) => {
            msg!("Instruction::Lock");
            match accounts {
                [system_program, rent, clock, spl_token, wallet, stake, stake_authority, token_account_source, token_account_stake_target, lock_account, ..] => {
                    lock(
                        system_program,
                        rent,
                        clock,
                        spl_token,
                        wallet,
                        stake,
                        stake_authority,
                        token_account_source,
                        token_account_stake_target,
                        lock_account,
                        input,
                    )
                }
                _ => Err(ProgramError::NotEnoughAccountKeys),
            }
        }

        Instruction::Unlock => {
            msg!("Instruction::unlock");
            match accounts {
                [clock, spl_token, wallet, stake, stake_authority, token_account_target, token_account_stake_source, lock_account, ..] => {
                    unlock(
                        clock,
                        spl_token,
                        wallet,
                        stake,
                        stake_authority,
                        token_account_target,
                        token_account_stake_source,
                        lock_account,
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
    stake: &AccountInfo<'a>,
    stake_authority: &AccountInfo<'a>,
    token_account: &AccountInfo<'a>,
    mint: &AccountInfo<'a>,
    input: &crate::instruction::InitializeStakeInput,
) -> ProgramResult {
    owner.is_signer()?;
    stake.is_signer()?;
    let (stake_authority_pubkey, bump_seed, token_account_pubkey) = derive_token_account(stake)?;

    is_derived(stake_authority_pubkey, stake_authority)?;
    is_derived(token_account_pubkey, token_account)?;

    let rent_state = Rent::from_account_info(rent)?;
    let lamports = rent_state.minimum_balance(ViewerStake::LEN);
    invoke::create_account(
        owner.clone(),
        stake.clone(),
        lamports,
        ViewerStake::LEN as u64,
        program_id,
        system_program,
    )?;
    let lamports = rent_state.minimum_balance(spl_token::state::Account::LEN);

    let authority_signature = ProgramPubkeySignature::new(stake, bump_seed);

    invoke::create_account_with_seed_signed(
        system_program,
        &owner,
        &token_account,
        stake_authority,
        "ViewerStake::token_account".to_string(),
        lamports,
        spl_token::state::Account::LEN as u64,
        &spl_token::id(),
        bump_seed,
        &authority_signature,
    )?;

    invoke::initialize_token_account_signed(
        token_account,
        &mint,
        &owner,
        rent,
        bump_seed,
        &authority_signature,
    )?;

    let mut state = stake.deserialize::<ViewerStake>()?;
    state.minimal_staking_time = input.minimal_staking_time;
    state.rank_requirements = input.rank_requirements.clone();
    state.owner = owner.pubkey();
    state.serialize_const(&mut *stake.try_borrow_mut_data()?)?;

    Ok(())
}

fn derive_token_account(stake: &AccountInfo) -> Result<(Pubkey, u8, Pubkey), ProgramError> {
    let (stake_authority_pubkey, bump_seed) = derive_stake_authority_account(stake);
    let derived = Pubkey::create_with_seed(
        &stake_authority_pubkey,
        "ViewerStake::token_account",
        &spl_token::id(),
    )?;
    Ok((stake_authority_pubkey, bump_seed, derived))
}

fn derive_stake_authority_account(stake: &AccountInfo) -> (Pubkey, u8) {
    let (stake_authority_pubkey, bump_seed) =
        Pubkey::find_program_address_for_pubkey(&stake.pubkey(), &crate::program_id());
    (stake_authority_pubkey, bump_seed)
}

fn lock<'a>(
    system_program: &AccountInfo<'a>,
    rent: &AccountInfo<'a>,
    clock: &AccountInfo<'a>,
    spl_token: &AccountInfo<'a>,
    wallet: &AccountInfo<'a>,
    stake: &AccountInfo<'a>,
    stake_authority: &AccountInfo<'a>,
    token_account_source: &AccountInfo<'a>,
    token_account_stake_target: &AccountInfo<'a>,
    lock_account: &AccountInfo<'a>,
    input: crate::instruction::LockInput,
) -> ProgramResult {
    let state = stake.deserialize::<ViewerStake>()?;
    let clock = Clock::from_account_info(clock)?;
    if input.duration < state.rank_requirements[0].minimal_staking_time {
        return crate::error::Error::LockStakingTimeMustBeMoreThanMinimal.into();
    }
    let (stake_authority_pubkey, bump_seed, token_account_pubkey) = derive_token_account(stake)?;

    let (lock_account_pubkey, seed) = Pubkey::create_with_seed_for_pubkey(
        &stake_authority_pubkey,
        &wallet.pubkey(),
        &program_id(),
    )?;

    is_derived(stake_authority_pubkey, stake_authority)?;
    is_derived(token_account_pubkey, token_account_stake_target)?;
    is_derived(lock_account_pubkey, lock_account)?;

    let authority_signature = ProgramPubkeySignature::new(stake, bump_seed);

    let mut lock_state = if stake.data_is_empty() {
        let lock_state = ViewerLock {
            amount: input.amount,
            owner: wallet.pubkey(),
            locked_until: clock.unix_timestamp + input.duration,
            version: StateVersion::V1,
        };
        let rent_state = Rent::from_account_info(rent)?;
        let lamports = rent_state.minimum_balance(ViewerLock::LEN);
        invoke::create_account_with_seed_signed(
            system_program,
            &wallet,
            &lock_account,
            stake_authority,
            seed,
            lamports,
            ViewerLock::LEN as u64,
            &program_id(),
            bump_seed,
            &authority_signature,
        )?;
        lock_state
    } else {
        let mut lock_state = lock_account.deserialize::<ViewerLock>()?;
        is_derived(lock_state.owner, wallet)?;
        lock_state.locked_until = clock.unix_timestamp + input.duration;
        lock_state.amount += input.amount;
        lock_state
    };

    invoke::spl_token_transfer(
        spl_token,
        token_account_source,
        token_account_stake_target,
        wallet,
        input.amount,
    );
    lock_state.serialize_const(&mut *lock_account.try_borrow_mut_data()?)?;

    Ok(())
}

fn unlock<'a>(
    clock: &AccountInfo<'a>,
    spl_token: &AccountInfo<'a>,
    wallet: &AccountInfo<'a>,
    stake: &AccountInfo<'a>,
    stake_authority: &AccountInfo<'a>,
    token_account_target: &AccountInfo<'a>,
    token_account_stake_source: &AccountInfo<'a>,
    lock_account: &AccountInfo<'a>,
) -> ProgramResult {
    let mut lock_state = lock_account.deserialize::<ViewerLock>()?;

    is_derived(lock_state.owner, wallet)?;
    wallet.is_signer()?;
    let clock = Clock::from_account_info(clock)?;
    if lock_state.locked_until < clock.unix_timestamp {
        return crate::error::Error::UnlockCanBeDoneOnlyAfterStakeTimeLapsed.into();
    }

    let (stake_authority_pubkey, bump_seed, token_account_pubkey) = derive_token_account(stake)?;
    is_derived(token_account_pubkey, token_account_stake_source)?;

    let (lock_account_pubkey, seed) = Pubkey::create_with_seed_for_pubkey(
        &stake_authority_pubkey,
        &wallet.pubkey(),
        &program_id(),
    )?;

    is_derived(lock_account_pubkey, lock_account)?;

    let authority_signature = ProgramPubkeySignature::new(stake, bump_seed);

    invoke::spl_token_transfer_signed(
        spl_token,
        token_account_stake_source,
        token_account_target,
        stake_authority,
        lock_state.amount,
        &authority_signature,
    );
    lock_state.amount = 0;
    lock_state.serialize_const(&mut *lock_account.try_borrow_mut_data()?)?;

    Ok(())
}
