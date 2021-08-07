use std::borrow::Borrow;
use std::error::Error;

use sator_sdk::invoke::{self, ProgramPubkeySignature};
use sator_sdk::borsh::*;
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

use crate::instructions::Instruction;
use crate::state::*;
use borsh::{BorshSerialize, BorshDeserialize};


// Program entrypoint's implementation
pub fn process_instruction(
    program_id: &ProgramPubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    //let instruction = BorshDeserializeConst::<crate::instructions::Instruction>::deserialize_const(instruction_data)?;
    let instruction = Instruction::deserialize_const(instruction_data)?;
    match instruction {
        Instruction::InitializeShow(input) => {
            match accounts {
                [
                    system_program,
                    sysvar_rent,
                    spl_token_program,
                    owner,
                    show,
                    show_authority,
                    token_account,
                    mint,               
                ..] => initialize_show(program_id, 
                    system_program,
                    sysvar_rent,
                    spl_token_program,
                    owner,
                    show,
                    show_authority,
                    token_account,
                    mint,
                    input),
                _ => Err(ProgramError::NotEnoughAccountKeys)
            }
            
        }
        Instruction::InitializeViewer(_) => todo!(),
        Instruction::InitializeQuiz(_) => todo!(),
        Instruction::Claim => todo!(),
    }
}

fn initialize_show<'a>(program_id: &Pubkey, system_program: &AccountInfo<'a>, sysvar_rent: &AccountInfo<'a>, spl_token_program: &AccountInfo<'a>, owner: &AccountInfo<'a>, show: &AccountInfo<'a>, show_authority: &AccountInfo<'a>, token_account: &AccountInfo<'a>, mint: &AccountInfo<'a>, input: crate::instructions::InitializeShowInput)-> ProgramResult {
    let (show_authority_pubkey, bump_seed) = Pubkey::find_program_address_for_pubkey(&show.pubkey(), program_id);
    let token_account_pubkey = Pubkey::create_with_seed(
        &show_authority_pubkey,
        Show::token_account,
        &spl_token::id(),
    )?;   
    is_derived(show_authority_pubkey, show_authority)?;
    is_derived(show_authority_pubkey, show_authority)?;
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