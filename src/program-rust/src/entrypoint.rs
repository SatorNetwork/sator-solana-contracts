#![cfg(all(target_arch = "bpf", not(feature = "no-entrypoint")))]

use solana_program::program_error::{PrintProgramError, ProgramError};
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, pubkey::Pubkey,
};

use crate::processor::process_instruction;

// Declare and export the program's entrypoint
entrypoint!(process_instruction);
