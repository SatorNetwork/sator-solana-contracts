#![feature(trivial_bounds)]
//! Stake pool for viewers.
//! - `StakePool.owner` can create stake pool, `owner` onwards.
//! - `owner` specificities if viewer KYC required to participate in staking.
//! - Only `owner` can added KYC to `ViewerStakeAccount`. KYC is just flag, not detailed description of how she was KYCed.
//! - `owner` specifies minimal amount to stake and minimal time to stake. Up to 4 stake `ranks`.
//! - For each `rank` `owner` specifies APY `reward_multiplier`. So `reward = stake time / year * amount * reward_multiplier`.
//! - Default `reward_multiplier` is zero. `owner` can update `reward_multiplier` on `StakePool`. Existing `ViewerStakeAccount` rewards cannot be updated until unstake.
//! - Viewer should stake at least minimal amount and time to stake to fit specified rank.
//! - Viewer can stake any amount, so minimal time should be not less than minimal for smallest rank.
//! - Staked amount can be less than minimal rank requirement.
//! - `owner` must top up stake `StakePool.token_account` to allow non zero `reward_multiplier`s.
//! - Adding amount resets the timer to zero. Non resetting option is possible, but need to be discussed if need to be implemented now..
//! - Allowing rewards depending on count of passed quizzes is possible, but should be discussed if should be implemented now.
//! - Anybody can stake for Viewer if it does not resets its stake rank. Only Viewer can add to stake if it resets stake rank.
//! - `Claim` can be done by `Viewer` signature to any address or via permissionless call to `associated token account` for SAO mint on `Viewer` wallet (`ViewerStake.owner`).
//! - `TypeScript` sdk to call on chain program is coded with example to call each instruction.
//! - So `ViewerStake` is operation under `owner` or `Viewer` credentials.
//! - `stake_authority` is derived operations are signed by on chain stake derived signature
//! - `stake_pool`'s `token_account` is derived from and owned by `stake_authority`
//! - `stake_viewer_account` for each user is derived from wallet and pool
//!
//!```rust, ignore
//! let stake_pool = Pubkey::new_unique();
//! let (stake_authority, _) = Pubkey::find_program_address(&[&stake_pool.to_bytes()[..32]], &stake_viewer_program_id());
//! let token_account_stake_pool_target = Pubkey::create_with_seed(
//!     &stake_authority,
//!     "ViewerStakePool::token_account",
//!     &spl_token::id(),
//! );
//! // Pubkey.to_string is longer than 32 chars limit in Solana for seed
//! // ETH compatible something
//! let seed = user_wallet.to_bytes();
//! let seed = bs58::encode(&seed[..20]).into_string();
//! let viewer_stake_account = Pubkey::create_with_seed(stake_authority, &seed, &stake_viewer_program_id());
//!```

pub mod entrypoint;
pub mod errors;
pub mod instruction;
pub mod processor;
pub mod state;
#[cfg(all(feature = "test-bpf", test))]
mod tests;
pub mod types;

//#[cfg(any(all(test, target_arch = "bpf"), all(not(test), not(target_arch = "bpf") )))]
//#[cfg(all(feature = "test-bpf", test))]
#[cfg(test)]
mod transactions;

#[cfg(test)]
mod tests_helpers;

use sator_sdk::types::ProgramPubkey;

solana_program::declare_id!("CL9tjeJL38C3eWqd6g7iHMnXaJ17tmL2ygkLEHghrj4u");

pub fn stake_viewer_program_id() -> ProgramPubkey {
    crate::id()
}
