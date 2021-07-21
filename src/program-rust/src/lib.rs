#![feature(trivial_bounds)]

mod state;
#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;
mod processor;
mod types;
mod instruction;
mod sdk;

#[cfg(all(feature = "test-bpf", test))]
mod tests;

//#[cfg(any(all(test, target_arch = "bpf"), all(not(test), not(target_arch = "bpf") )))]
//#[cfg(all(feature = "test-bpf", test))]
#[cfg(test)]
mod transactions;

#[cfg(test)]
mod spl_transactions;


use sdk::types::ProgramPubkey;
use solana_program::{account_info::AccountInfo, pubkey::Pubkey};
use state::*;


solana_program::declare_id!("2ALZgMNre2qynTTyxWtgWG6L2L56n39aBGegS1yvxwya");

pub fn program_id() -> ProgramPubkey {
    crate::id()
}