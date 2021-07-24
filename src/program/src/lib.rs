#![feature(trivial_bounds)]

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;
mod error;
mod instruction;
mod processor;
mod sdk;
mod state;
mod types;

#[cfg(all(feature = "test-bpf", test))]
mod tests;

//#[cfg(any(all(test, target_arch = "bpf"), all(not(test), not(target_arch = "bpf") )))]
//#[cfg(all(feature = "test-bpf", test))]
#[cfg(test)]
mod transactions;

#[cfg(test)]
mod spl_transactions;
#[cfg(test)]
mod tests_helpers;

use sdk::types::ProgramPubkey;
use solana_program::{account_info::AccountInfo, pubkey::Pubkey};
use state::*;

solana_program::declare_id!("2ALZgMNre2qynTTyxWtgWG6L2L56n39aBGegS1yvxwya");

pub fn program_id() -> ProgramPubkey {
    crate::id()
}
