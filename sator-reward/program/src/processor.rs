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

use crate::instruction::{InitializeQuizInput, InitializeViewerInput, Instruction};
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
            [system_program, sysvar_rent, clock, owner, show, show_authority, quiz, ..] => {
                let winners = accounts.iter().skip(7);
                initialize_quiz(
                    program_id,
                    system_program,
                    sysvar_rent,
                    clock,
                    owner,
                    show,
                    show_authority,
                    quiz,
                    winners,
                    input,
                )
            }
            _ => Err(ProgramError::NotEnoughAccountKeys),
        },
        Instruction::Claim => match accounts {
            [spl_token, show_owner, show, show_authority, user_wallet_winner, show_token_account, user_token_account, ..] =>
            {
                let quizes = accounts.iter().skip(7);
                claim(
                    program_id,
                    spl_token,
                    show_owner,
                    show,
                    show_authority,
                    user_wallet_winner,
                    show_token_account,
                    user_token_account,
                    quizes,
                )
            }
            _ => Err(ProgramError::NotEnoughAccountKeys),
        },
    }
}

fn claim<'a>(
    program_id: &Pubkey,
    spl_token: &AccountInfo<'a>,
    show_owner: &AccountInfo<'a>,
    show: &AccountInfo<'a>,
    show_authority: &AccountInfo<'a>,
    winner: &AccountInfo<'a>,
    show_token_account: &AccountInfo<'a>,
    user_token_account: &AccountInfo<'a>,
    quizes: std::iter::Skip<std::slice::Iter<AccountInfo<'a>>>,
) -> ProgramResult {
    let mut show_state = show.deserialize::<Show>()?;
    show_state.initialized()?;
    is_owner!(show_owner, show_state);
    show_owner.is_signer()?;

    let (show_authority_pubkey, bump_seed) =
        Pubkey::find_program_address_for_pubkey(&show.pubkey(), program_id);
    let authority_signature = ProgramPubkeySignature::new(show, bump_seed);

    for quiz in quizes {
        let mut quiz_state = quiz.deserialize::<Quiz>()?;
        let (quiz_pubkey, _) = Pubkey::create_with_seed_index(
            &show_authority_pubkey,
            Show::QUIZES,
            quiz_state.index as u64,
            &program_id,
        )?;
        is_derived(quiz_pubkey, quiz)?;
        quiz.is_owner(program_id)?;
        let total_points: u32 = quiz_state.winners.iter().map(|x| x.points).sum();

        if let Some(winner) = quiz_state
            .winners
            .iter_mut()
            .filter(|x| x.user_wallet == winner.pubkey() && !x.claimed)
            .next()
        {
            winner.claimed = true;

            let amount = quiz_state
                .amount
                .checked_mul(winner.points as u64)
                .ok_or(ProgramError::Custom(crate::errors::Error::Overflow.into()))?
                / total_points as u64;

            invoke::spl_token_transfer_signed(
                spl_token,
                show_token_account,
                user_token_account,
                show_authority,
                amount,
                &authority_signature,
            )?;
        }
        quiz_state.serialize_const(&mut *quiz.try_borrow_mut_data()?)?;
    }

    // possibly burn quiz if it was last here if it was latest winner claim, for now doing nothing as winners can grab value for a long or use winner board for reference

    Ok(())
}

fn initialize_quiz<'a>(
    program_id: &Pubkey,
    system_program: &AccountInfo<'a>,
    sysvar_rent: &AccountInfo<'a>,
    clock: &AccountInfo<'a>,
    owner: &AccountInfo<'a>,
    show: &AccountInfo<'a>,
    show_authority: &AccountInfo<'a>,
    quiz: &AccountInfo<'a>,
    viewers: std::iter::Skip<std::slice::Iter<AccountInfo<'a>>>,
    input: InitializeQuizInput,
) -> ProgramResult {
    let mut show_state = show.deserialize::<Show>()?;
    show_state.initialized()?;

    owner.is_signer()?;
    is_owner!(owner, show_state);

    let (show_authority_pubkey, bump_seed) =
        Pubkey::find_program_address_for_pubkey(&show.pubkey(), program_id);
    let authority_signature = ProgramPubkeySignature::new(show, bump_seed);
    let (quiz_pubkey, seed) = Pubkey::create_with_seed_index(
        &show_authority_pubkey,
        Show::QUIZES,
        show_state.quizes_index as u64,
        &program_id,
    )?;
    is_derived(show_authority_pubkey, show_authority)?;
    is_derived(quiz_pubkey, quiz)?;

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

    let winners_pubkeys: Vec<_> = input.winners.iter().map(|x| x.owner).collect();

    let mut quiz_state = quiz.deserialize::<Quiz>()?;
    let winners_state: Vec<_> = input
        .winners
        .into_iter()
        .enumerate()
        .map(|(i, x)| Winner {
            claimed: false,
            user_wallet: winners_pubkeys[i],
            points: x.points,
        })
        .collect();
    quiz_state.winners = <_>::default();
    for (i, winner) in winners_state.into_iter().enumerate() {
        quiz_state.winners[i] = winner;
    }
    quiz_state.uninitialized()?;
    quiz_state.index = show_state.quizes_index;
    quiz_state.amount = input.amount;
    quiz_state.version = StateVersion::V1;
    for (i, viewer) in viewers.into_iter().enumerate() {
        let (viewer_pubkey, _) = Pubkey::create_with_seed_for_pubkey(
            &show_authority_pubkey,
            &winners_pubkeys[i],
            &program_id,
        )?;
        viewer.is_owner(program_id)?;
        let viewer = viewer.deserialize::<Viewer>()?;
        viewer.initialized()?;
    }

    let clock = Clock::from_account_info(clock)?;
    quiz_state.locked_until = clock.unix_timestamp + show_state.lock_time;
    quiz_state.serialize_const(&mut *quiz.try_borrow_mut_data()?)?;

    show_state.quizes_index += 1;
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
    show_state.initialized()?;
    is_owner!(owner, show_state);

    let (show_authority_pubkey, bump_seed) =
        Pubkey::find_program_address_for_pubkey(&show.pubkey(), program_id);
    let (viewer_pubkey, seed) =
        Pubkey::create_with_seed_for_pubkey(&show_authority_pubkey, &input.user, &program_id)?;

    is_derived(show_authority_pubkey, show_authority)?;
    is_derived(viewer_pubkey, viewer)?;

    let authority_signature = ProgramPubkeySignature::new(show, bump_seed);
    let viewer_state = Viewer {
        version: StateVersion::V1,
    };
    let rent_state = Rent::from_account_info(sysvar_rent)?;
    let lamports = rent_state.minimum_balance(Viewer::LEN);

    invoke::create_account_with_seed_signed(
        system_program,
        owner,
        viewer,
        show_authority,
        &seed[..],
        lamports,
        Viewer::LEN as u64,
        program_id,
        &authority_signature,
    )?;

    viewer_state.serialize_const(&mut *viewer.try_borrow_mut_data()?)?;

    Ok(())
}

fn initialize_show<'a>(
    program_id: &Pubkey,
    system_program: &AccountInfo<'a>,
    sysvar_rent: &AccountInfo<'a>,
    spl_token_program: &AccountInfo<'a>,
    // fee_payer
    owner: &AccountInfo<'a>,
    show: &AccountInfo<'a>,
    show_authority: &AccountInfo<'a>,
    token_account: &AccountInfo<'a>,
    mint: &AccountInfo<'a>,
    input: crate::instruction::InitializeShowInput,
) -> ProgramResult {
    let (show_authority_pubkey, bump_seed) =
        Pubkey::find_program_address_for_pubkey(&show.pubkey(), program_id);
    let token_account_pubkey = Pubkey::create_with_seed(
        &show_authority_pubkey,
        Show::TOKEN_ACCOUNT,
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
        spl_token_program,
    )?;

    let authority_signature = ProgramPubkeySignature::new(show, bump_seed);

    let lamports = rent_state.minimum_balance(spl_token::state::Account::LEN);
    invoke::create_account_with_seed_signed(
        system_program,
        owner,
        token_account,
        show_authority,
        Show::TOKEN_ACCOUNT,
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
    state.uninitialized()?;
    state.lock_time = input.reward_lock_time;
    state.owner = owner.pubkey();
    state.version = StateVersion::V1;
    state.serialize_const(&mut *show.try_borrow_mut_data()?)?;

    Ok(())
}
