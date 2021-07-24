use std::borrow::Borrow;

use solana_program::msg;
use solana_program::program_error::{PrintProgramError, ProgramError};
use solana_program::program_pack::Pack;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, pubkey::Pubkey,
};

use crate::instruction::Instruction;
use crate::sdk::program::{wire, AccountPatterns, PubkeyPatterns};
use crate::sdk::types::ProgramPubkey;
use crate::sdk::{
    borsh::{BorshDeserializeConst, BorshSerializeConst},
    invoke,
};
use crate::state::{StateVersion, ViewerStake};
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
                [clock, spl_token, wallet, stake, stake_authority, token_account_source, token_account_stake_target, lock_account, ..] => {
                    lock(
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
        _ => todo!("not implemented yet"),
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

    wire(stake_authority_pubkey, stake_authority)?;
    wire(token_account_pubkey, token_account)?;

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

    let authority_signature = [&stake.pubkey().to_bytes()[..32], &[bump_seed]];
    let authority_signature = &[&authority_signature[..]];

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
        authority_signature,
    )?;

    invoke::initialize_token_account_signed(
        token_account,
        &mint,
        &owner,
        rent,
        bump_seed,
        authority_signature,
    )?;

    let x= stake.try_borrow_data().expect("no borrow issues");
    let mut state = stake.deserialize::<ViewerStake>()?;
    state.minimal_staking_time = input.minimal_staking_time;
    state.rank_requirements = input.rank_requirements.clone();
    state.owner = owner.pubkey();
    state.serialize_const(&mut *stake.try_borrow_mut_data()?)?;

    Ok(())
}

fn derive_token_account(stake: &AccountInfo) -> Result<(Pubkey, u8, Pubkey), ProgramError> {
    let (stake_authority_pubkey, bump_seed) =
        Pubkey::find_program_address_for_pubkey(&stake.pubkey(), &crate::program_id());
    let token_account_pubkey = Pubkey::create_with_seed(
        &stake_authority_pubkey,
        "ViewerStake::token_account",
        &spl_token::id(),
    )?;
    Ok((stake_authority_pubkey, bump_seed, token_account_pubkey))
}

fn lock(
    clock: &AccountInfo,
    spl_token: &AccountInfo,
    wallet: &AccountInfo,
    stake: &AccountInfo,
    stake_authority: &AccountInfo,
    token_account_source: &AccountInfo,
    token_account_stake_target: &AccountInfo,
    lock_account: &AccountInfo,
    input: crate::instruction::LockInput,
) -> ProgramResult {
    wallet.is_signer()?;

    let state = stake.deserialize::<ViewerStake>()?;
    
    let (stake_authority_pubkey, bump_seed, token_account_pubkey) = derive_token_account(stake)?;
    
    wire(stake_authority_pubkey, stake_authority)?;
    wire(token_account_pubkey, token_account_stake_target)?;

    Ok(())
    
}
