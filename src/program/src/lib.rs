#![feature(trivial_bounds)]
//! Stake for viewers:
//! - user stakes has minimal amounts  to stake and time to stake to fit specified rank, which is associate with multiplier
//! - user can stake any amount, so minimal time should be not less than minimal for smallest rank
//! - amount can be less than minimal rank
//! - adding amount resets the timer
//! - total 4 ranks
//! - lock accounts and stake token account are derived - operations are signed by on chain stake derived signature
//!
pub mod entrypoint;
#[cfg(not(feature = "no-entrypoint"))]
pub mod error;
pub mod instruction;
mod processor;
mod sdk;
pub mod state;
pub mod types;

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
