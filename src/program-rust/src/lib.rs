#![feature(trivial_bounds)]

mod state;
mod entrypoint;
mod types;
mod instruction;
mod sdk;

use sdk::types::ProgramPubkey;
use solana_program::{account_info::AccountInfo, pubkey::Pubkey};
use state::*;


solana_program::declare_id!("2ALZgMNre2qynTTyxWtgWG6L2L56n39aBGegS1yvxwya");

pub fn program_id() -> ProgramPubkey {
    crate::id()
}