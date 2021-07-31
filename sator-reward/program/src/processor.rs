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

use crate::state::*;
use borsh::BorshSerialize;

// Program entrypoint's implementation
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    todo!()
}