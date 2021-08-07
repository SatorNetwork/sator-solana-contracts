#![feature(trivial_bounds)]

#[cfg(all(target_arch = "bpf", not(feature = "no-entrypoint")))]
mod entrypoint;
pub mod errors;
pub mod instructions;
mod processor;
pub mod state;
pub mod types;

use sator_sdk::types::ProgramPubkey;
use solana_program::{account_info::AccountInfo, pubkey::Pubkey};

solana_program::declare_id!("CL9tjeJL38C3eWqd6g7iHMnXaJ17tmL2ygkLEHghrj4u");

pub fn program_id() -> ProgramPubkey {
    crate::id()
}
