use std::borrow::Borrow;
use std::collections::hash_map::{DefaultHasher, RandomState};
use std::collections::{HashMap, HashSet};
use std::convert::TryInto;
use std::error::Error;
use std::hash::BuildHasher;

use sator_sdk::borsh::*;
use sator_sdk::invoke::{self, ProgramPubkeySignature};
use sator_sdk::is_owner;
use sator_sdk::program::*;
use sator_sdk::state::StateVersion;
use sator_sdk::types::*;
use solana_program::clock::Clock;
use solana_program::msg;
use solana_program::program_error::{PrintProgramError, ProgramError};
use solana_program::program_pack::Pack;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, pubkey::Pubkey,
};

use crate::instructions::{InitializeQuizInput, InitializeViewerInput, Instruction};
use crate::state::*;
use crate::types::Winner;
use borsh::{BorshDeserialize, BorshSerialize};

// Program entrypoint's implementation
pub fn process_instruction(
    program_id: &ProgramPubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    //let instruction = BorshDeserializeConst::<crate::instructions::Instruction>::deserialize_const(instruction_data)?;
    let instruction = Instruction::deserialize_const(instruction_data)?;
    match instruction {
        Instruction::InitializeShow(input) => match accounts {
            [system_program, sysvar_rent, spl_token_program, owner, show, show_authority, token_account, mint, ..] => {
                initialize_show(
                    program_id,
                    system_program,
                    sysvar_rent,
                    spl_token_program,
                    owner,
                    show,
                    show_authority,
                    token_account,
                    mint,
                    input,
                )
            }
            _ => Err(ProgramError::NotEnoughAccountKeys),
        },
        Instruction::InitializeViewer(input) => match accounts {
            [system_program, sysvar_rent, owner, show, show_authority, viewer, ..] => {
                initialize_viewer(
                    program_id,
                    system_program,
                    sysvar_rent,
                    owner,
                    show,
                    show_authority,
                    viewer,
                    input,
                )
            }
            _ => Err(ProgramError::NotEnoughAccountKeys),
        },
        Instruction::InitializeQuiz(input) => match accounts {
            [system_program, sysvar_rent, clock, show, owner, show_authority, quiz, ..] => {
                let winners = accounts.iter().skip(6);
                initialize_winners(
                    program_id,
                    system_program,
                    sysvar_rent,
                    clock,
                    show,
                    owner,
                    show_authority,
                    quiz,
                    winners,
                    input,
                )
            }
            _ => Err(ProgramError::NotEnoughAccountKeys),
        },
        Instruction::Claim => todo!(),
    }
}

fn initialize_winners<'a>(
    program_id: &Pubkey,
    system_program: &AccountInfo<'a>,
    sysvar_rent: &AccountInfo<'a>,
    clock: &AccountInfo<'a>,
    show: &AccountInfo<'a>,
    owner: &AccountInfo<'a>,
    show_authority: &AccountInfo<'a>,
    quiz: &AccountInfo<'a>,
    winners: std::iter::Skip<std::slice::Iter<AccountInfo<'a>>>,
    input: InitializeQuizInput,
) -> ProgramResult {
    let mut show_state = show.deserialize::<Show>()?;

    show_state.uninitialized()?;
    is_owner!(owner, show_state);
    owner.is_signer()?;

    let (show_authority_pubkey, bump_seed) =
        Pubkey::find_program_address_for_pubkey(&show.pubkey(), program_id);
    let (quiz_pubkey, seed) = Pubkey::create_with_seed_index(
        &show_authority_pubkey,
        Show::quizes,
        show_state.quizes_len as u64,
        &program_id,
    )?;
    is_derived(show_authority_pubkey, show_authority)?;
    is_derived(quiz_pubkey, quiz)?;

    let authority_signature = ProgramPubkeySignature::new(show_authority, bump_seed);
    let rent_state = Rent::from_account_info(sysvar_rent)?;
    let lamports = rent_state.minimum_balance(Quiz::LEN);

    invoke::create_account_with_seed_signed(
        system_program,
        &owner,
        &quiz,
        show_authority,
        &seed[..],
        lamports,
        Quiz::LEN as u64,
        program_id,
        &authority_signature,
    )?;

    let mut quiz_state = quiz.deserialize::<Quiz>()?;
    quiz_state.initialized()?;
    quiz_state.winners = input.winners;
    let mut winners_pubkeys = Vec::new();
    for winner in quiz_state.winners.iter() {
        winners_pubkeys.push(winner.user_wallet);
    }

    for winner in winners {
        let (viewer_pubkey, _) = Pubkey::create_with_seed_for_pubkey(
            &show_authority_pubkey,
            &winner.pubkey(),
            &program_id,
        )?;
        let viewer = winner.deserialize::<Viewer>()?;
        winner.is_owner(program_id)?;
        viewer.initialized()?;

        /// small vec, so not using hash
        if !(winners_pubkeys.contains(&winner.pubkey())) {
            return Err(crate::errors::Error::InitializeQuizWinnerIsNotInList.into());
        }
    }

    let clock = Clock::from_account_info(clock)?;
    quiz_state.locked_until = clock.unix_timestamp + show_state.lock_time;
    quiz_state.serialize_const(&mut *quiz.try_borrow_mut_data()?)?;

    show_state.quizes_len += 1;
    show_state.serialize_const(&mut *show.try_borrow_mut_data()?)?;

    Ok(())
}

fn initialize_viewer<'a>(
    program_id: &Pubkey,
    system_program: &AccountInfo<'a>,
    sysvar_rent: &AccountInfo<'a>,
    owner: &AccountInfo<'a>,
    show: &AccountInfo<'a>,
    show_authority: &AccountInfo<'a>,
    viewer: &AccountInfo<'a>,
    input: InitializeViewerInput,
) -> ProgramResult {
    let show_state = show.deserialize::<Show>()?;
    show_state.uninitialized()?;
    is_owner!(owner, show_state);

    let (show_authority_pubkey, bump_seed) =
        Pubkey::find_program_address_for_pubkey(&show.pubkey(), program_id);
    let (viewer_pubkey, seed) = Pubkey::create_with_seed_for_pubkey(
        &show_authority_pubkey,
        &input.user_wallet,
        &program_id,
    )?;
    is_derived(show_authority_pubkey, show_authority)?;
    is_derived(viewer_pubkey, viewer)?;

    let authority_signature = ProgramPubkeySignature::new(show_authority, bump_seed);
    let stake_account_state = Viewer {
        version: StateVersion::V1,
    };
    let rent_state = Rent::from_account_info(sysvar_rent)?;
    let lamports = rent_state.minimum_balance(Viewer::LEN);

    invoke::create_account_with_seed_signed(
        system_program,
        &owner,
        &viewer,
        show_authority,
        &seed[..],
        lamports,
        Viewer::LEN as u64,
        program_id,
        &authority_signature,
    )?;

    Ok(())
}

fn initialize_show<'a>(
    program_id: &Pubkey,
    system_program: &AccountInfo<'a>,
    sysvar_rent: &AccountInfo<'a>,
    spl_token_program: &AccountInfo<'a>,
    owner: &AccountInfo<'a>,
    show: &AccountInfo<'a>,
    show_authority: &AccountInfo<'a>,
    token_account: &AccountInfo<'a>,
    mint: &AccountInfo<'a>,
    input: crate::instructions::InitializeShowInput,
) -> ProgramResult {
    let (show_authority_pubkey, bump_seed) =
        Pubkey::find_program_address_for_pubkey(&show.pubkey(), program_id);
    let token_account_pubkey = Pubkey::create_with_seed(
        &show_authority_pubkey,
        Show::token_account,
        &spl_token::id(),
    )?;
    is_derived(show_authority_pubkey, show_authority)?;
    is_derived(token_account_pubkey, token_account)?;
    show.is_signer()?;
    owner.is_signer()?;

    let rent_state = Rent::from_account_info(sysvar_rent)?;
    let lamports = rent_state.minimum_balance(Show::LEN);

    invoke::create_account(
        owner.clone(),
        show.clone(),
        lamports,
        Show::LEN as u64,
        program_id,
        system_program,
    )?;

    let authority_signature = ProgramPubkeySignature::new(show, bump_seed);

    let lamports = rent_state.minimum_balance(spl_token::state::Account::LEN);
    invoke::create_account_with_seed_signed(
        system_program,
        &owner,
        &token_account,
        show_authority,
        Show::token_account,
        lamports,
        spl_token::state::Account::LEN as u64,
        &spl_token::id(),
        &authority_signature,
    )?;

    invoke::initialize_token_account_signed(
        token_account,
        &mint,
        show_authority,
        sysvar_rent,
        &authority_signature,
    )?;

    let mut state = show.deserialize::<Show>()?;
    state.initialized()?;
    state.lock_time = input.reward_lock_time;
    state.owner = owner.pubkey();
    state.version = StateVersion::V1;
    state.serialize_const(&mut *show.try_borrow_mut_data()?)?;

    Ok(())
}
