#![feature(trivial_bounds)]
//! Reward for viewers.
//! 1. Creator initializes show
//! 2. Creator sets prove user can participate in quiz
//! 3. Quiz results are put into contract
//! 4. After some lock time, it is possible to claim reward from each winner quiz
//!
//! Derivation rules:
//!```rust, ignore
//! fn find_program_address_for_pubkey(
//!     seed: &Pubkey,
//!     program_id: &ProgramPubkey,
//! ) -> (ProgramDerivedPubkey, u8) {
//!     Pubkey::find_program_address(&[&seed.to_bytes()[..32]], program_id)
//! }
//! let show_authority = Pubkey::find_program_address_for_pubkey(&show.pubkey(), &program_id());
//! let token_account_show =
//!     Pubkey::create_with_seed(&show_authority.0, "Show::token_account", &spl_token::id())?;
//! let (show_authority_pubkey, _) = Pubkey::find_program_address_for_pubkey(show, &program_id());
//!         
//! let (show_authority_pubkey, _) =
//!     Pubkey::find_program_address_for_pubkey(&show.pubkey(), &program_id());
//! let (viewer_pubkey, _) = Pubkey::create_with_seed_for_pubkey(
//!     &show_authority_pubkey,
//!     &input.user_wallet,
//!     &program_id(),
//! )?;
//! fn create_with_seed_index(
//!     base: &Pubkey,
//!     seed: &str,
//!     seed_index: u64,
//!     owner: &ProgramPubkey,
//! ) -> Result<(ProgramDerivedPubkey, String), PubkeyError> {
//!     let seed = format!("{}{:?}", seed, seed_index);
//!     let pubkey = Pubkey::create_with_seed(base, &seed, owner)?;
//!     Ok((pubkey, seed))
//! }
//! let (quiz_pubkey, _) = Pubkey::create_with_seed_index(
//!     &show_authority_pubkey,
//!     "Show::quizes",
//!     show_quizes_index as u64,
//!     &program_id(),
//! )?;
//!```

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
pub mod transactions;

use sator_sdk::types::ProgramPubkey;
use solana_program::{account_info::AccountInfo, pubkey::Pubkey};

solana_program::declare_id!("DajevvE6uo5HtST4EDguRUcbdEMNKNcLWjjNowMRQvZ1");

pub fn program_id() -> ProgramPubkey {
    crate::id()
}
