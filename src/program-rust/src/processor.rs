use solana_program::program_error::{PrintProgramError, ProgramError};
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, pubkey::Pubkey,
};


// Program entrypoint's implementation
pub fn process_instruction(
    program_id: &Pubkey, 
    accounts: &[AccountInfo], 
    _instruction_data: &[u8], 
) -> ProgramResult {    
    todo!()
}