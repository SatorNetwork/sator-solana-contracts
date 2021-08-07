#![feature(trivial_bounds)]
//! Stake for viewers:
//! - user stakes has minimal amounts  to stake and time to stake to fit specified rank, which is associate with multiplier
//! - user can stake any amount, so minimal time should be not less than minimal for smallest rank
//! - amount can be less than minimal rank
//! - adding amount resets the timer
//! - total 4 ranks
//! - `stake_authority` is derived operations are signed by on chain stake derived signature
//! - `stake_pool`'s `token_account` is derived from and owned by `stake_authority`
//! - `stake_account` for each user is derived
//!```rust, ignore
//! let stake_pool = Pubkey::new_unique();
//! let (stake_authority, _) = Pubkey::find_program_address(&[&stake_pool.to_bytes()[..32]], &stake_viewer_program_id());
//! let token_account_stake_target = Pubkey::create_with_seed(
//!     &stake_authority,
//!     "ViewerStakePool::token_account",
//!     &spl_token::id(),
//! );
//! // Pubkey.to_string is longer than 32 chars limit in Solana for seed
//! // ETH compatible something
//! let seed = user_wallet.to_bytes();
//! let seed = bs58::encode(&seed[..20]).into_string();
//! let stake_account = Pubkey::create_with_seed(stake_authority, &seed, &stake_viewer_program_id());
//!```

pub mod entrypoint;
pub mod errors;
pub mod instruction;
mod processor;
pub mod state;
#[cfg(all(feature = "test-bpf", test))]
mod tests;
pub mod types;

//#[cfg(any(all(test, target_arch = "bpf"), all(not(test), not(target_arch = "bpf") )))]
//#[cfg(all(feature = "test-bpf", test))]
#[cfg(test)]
mod transactions;

#[cfg(test)]
mod spl_transactions;
#[cfg(test)]
mod tests_helpers;

use sator_sdk::types::ProgramPubkey;
use solana_program::{account_info::AccountInfo, pubkey::Pubkey};

solana_program::declare_id!("CL9tjeJL38C3eWqd6g7iHMnXaJ17tmL2ygkLEHghrj4u");

pub fn stake_viewer_program_id() -> ProgramPubkey {
    crate::id()
}
