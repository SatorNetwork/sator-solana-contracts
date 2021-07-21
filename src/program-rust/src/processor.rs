use solana_program::msg;
use solana_program::program_error::{PrintProgramError, ProgramError};
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, pubkey::Pubkey,
};

use crate::instruction::Instruction;
use crate::sdk::program::PubkeyPatterns;
use crate::sdk::types::ProgramPubkey;
use crate::sdk::{borsh::BorshDeserializeConst, invoke};
use crate::state::ViewerStake;
use borsh::*;

// Program entrypoint's implementation
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("process_instruction");
    return Ok(());
    // let instruction = Instruction::deserialize_const(instruction_data)?; //BorshDeserializeConst::<Instruction>::deserialize_const(instruction_data)?;
    // match instruction {
    //     Instruction::InitializeStake(input) => {
    //         msg!("Instruction::InitializeStake");
    //         match accounts {
    //             [rent, spl_token, owner, stake, stake_authority, token_account, ..] => {
    //                 initialize_stake(
    //                     program_id,
    //                     rent,
    //                     spl_token,
    //                     owner,
    //                     stake,
    //                     stake_authority,
    //                     token_account,
    //                     &input,
    //                 )
    //             }
    //             _ => Err(ProgramError::NotEnoughAccountKeys),
    //         }
    //     }
    //     _ => todo!(),
    // }
}

fn initialize_stake<'a>(
    program_id: &ProgramPubkey,
    rent: &AccountInfo<'a>,
    spl_token: &AccountInfo<'a>,
    owner: &AccountInfo<'a>,
    stake: &AccountInfo<'a>,
    stake_authority: &AccountInfo<'a>,
    token_account: &AccountInfo<'a>,
    input: &crate::instruction::InitializeStakeInput,
) -> ProgramResult {
    //let rent = Rent::from_account_info(rent)?;
    //let lamports = rent.minimum_balance(ViewerStake::LEN);
    //invoke::create_account(owner.clone(), stake.clone(), lamports, ViewerStake::LEN as u64, program_id);

    Ok(())
}
