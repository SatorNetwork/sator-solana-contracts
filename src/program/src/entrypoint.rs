#![cfg(all(target_arch = "bpf", not(feature = "no-entrypoint")))]

use solana_program::msg;
use solana_program::program_error::{PrintProgramError, ProgramError};
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, pubkey::Pubkey,
};

// Declare and export the program's entrypoint
entrypoint!(process_instruction);
fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    if let Err(error) =
        crate::processor::process_instruction(program_id, accounts, instruction_data)
    {
        msg!("{:?}", error);
        return Err(error);
    }
    Ok(())
}
