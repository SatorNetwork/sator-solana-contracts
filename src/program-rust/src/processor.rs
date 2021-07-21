use solana_program::msg;
use solana_program::program_error::{PrintProgramError, ProgramError};
use solana_program::program_pack::Pack;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, pubkey::Pubkey,
};

use crate::instruction::Instruction;
use crate::sdk::program::{AccountPatterns, PubkeyPatterns, wire};
use crate::sdk::types::ProgramPubkey;
use crate::sdk::{borsh::{BorshDeserializeConst, BorshSerializeConst}, invoke};
use crate::state::{StateVersion, ViewerStake};
use borsh::*;

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
    
    let (stake_authority_pubkey, bump_seed) = Pubkey::find_program_address_for_pubkey(&stake.pubkey(), &crate::program_id());
    let token_account_pubkey = Pubkey::create_with_seed(
        &stake_authority_pubkey,
        "ViewerStake::token_account",
        &spl_token::id(),
    )?;

    wire(stake_authority_pubkey, stake_authority)?;
    wire(token_account_pubkey, token_account)?;


    let rent_state = Rent::from_account_info(rent)?;
    let lamports = rent_state.minimum_balance(ViewerStake::LEN);
    invoke::create_account(owner.clone(), stake.clone(), lamports, ViewerStake::LEN as u64, program_id, system_program)?;
    let lamports = rent_state.minimum_balance(spl_token::state::Account::LEN);
    msg!("a312312312312312313123adadasdasdasdasdsadasdsadasdasdsa");    
    
    let authority_signature = [&stake.pubkey().to_bytes()[..32], &[bump_seed]];
    let authority_signature = &[&authority_signature[..]];    
    msg!("1:   {:?}", token_account.pubkey());
    msg!("1:   {:?}", token_account_pubkey);
    invoke::create_account_with_seed_signed(
        &owner,
         &token_account,
         stake_authority,
         "ViewerStake::token_account".to_string(),
          lamports, 
          spl_token::state::Account::LEN as u64, 
          &spl_token::id(), 
          bump_seed,
          authority_signature)?;
    
    msg!("qweqweqweqwe");    
    
    invoke::initialize_token_account_signed(token_account, &mint, &owner, rent, bump_seed, authority_signature)?;

    let mut state = ViewerStake::try_from_slice(&stake.data.borrow())?;
    state.minimal_staking_time = input.minimal_staking_time;
    state.rank_requirements = input.rank_requirements.clone();    
    state.owner = owner.pubkey();
    state.serialize_const(&mut *stake.try_borrow_mut_data()?)?;

    msg!("asdadasdasdd");    


    Ok(())
}
