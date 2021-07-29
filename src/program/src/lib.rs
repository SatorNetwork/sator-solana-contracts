#![feature(trivial_bounds)]
//! Stake for viewers:
//! - user stakes has minimal amounts  to stake and time to stake to fit specified rank, which is associate with multiplier
//! - user can stake any amount, so minimal time should be not less than minimal for smallest rank
//! - amount can be less than minimal rank
//! - adding amount resets the timer
//! - total 4 ranks
//! - `stake_authority` is derived operations are signed by on chain stake derived signature
//! - `stake_pool` `token_account` is derived from and owned by `stake_authority`
//! - `stake_account` for each user is derived 
pub mod entrypoint;
pub mod errors;
pub mod instruction;
mod processor;
pub mod sdk;
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

solana_program::declare_id!("CL9tjeJL38C3eWqd6g7iHMnXaJ17tmL2ygkLEHghrj4u");

pub fn program_id() -> ProgramPubkey {
    crate::id()
}
