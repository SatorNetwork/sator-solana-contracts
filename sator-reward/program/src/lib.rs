#![feature(trivial_bounds)]

#[cfg(all(target_arch = "bpf", not(feature = "no-entrypoint")))]
mod entrypoint;
pub mod errors;
pub mod instruction;
mod processor;
pub mod state;
#[cfg(all(feature = "test-bpf", test))]
mod tests;
#[cfg(all(feature = "test-bpf", test))]
mod tests_helpers;
pub mod types;

#[cfg(all(feature = "test-bpf", test))]
mod transactions;

use sator_sdk::types::ProgramPubkey;
use solana_program::{account_info::AccountInfo, pubkey::Pubkey};

solana_program::declare_id!("DajevvE6uo5HtST4EDguRUcbdEMNKNcLWjjNowMRQvZ1");

pub fn program_id() -> ProgramPubkey {
    crate::id()
}
