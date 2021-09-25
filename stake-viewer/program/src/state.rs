//! Program owned state

use std::ops::Mul;
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use sator_sdk::state::StateVersion;
use sator_sdk::types::{ApproximateSeconds, SignerPubkey, TokenAmount};
use solana_program::clock::UnixTimestamp;
use solana_program::pubkey::Pubkey;
use solana_program::{entrypoint::ProgramResult, program_error::ProgramError};

use crate::types::*;

/// Pool state and rules
#[repr(C)]
#[derive(Debug, BorshDeserialize, BorshSerialize, BorshSchema, Default)]
pub struct ViewerStakePool {
    pub version: StateVersion,
    /// ranks 0 consider to have minimal time
    pub ranks: [Rank; 4],
    // can initialize state and change rules
    pub owner: SignerPubkey,
}
/// User stake account state
#[repr(C)]
#[derive(Debug, BorshDeserialize, BorshSerialize, BorshSchema, Default)]
pub struct ViewerStake {
    pub version: StateVersion,
    pub staked_at: UnixTimestamp,
    pub staked_until: UnixTimestamp,
    /// user owner of stake
    pub owner: SignerPubkey,
    pub amount: TokenAmount,
}

impl ViewerStake {
    pub const LEN: usize = 57;

    pub fn uninitialized(&self) -> ProgramResult {
        if self.version == StateVersion::Uninitialized {
            Ok(())
        } else {
            Err(ProgramError::AccountAlreadyInitialized)
        }
    }
    /// Error if not initialized
    pub fn initialized(&self) -> ProgramResult {
        if self.version != StateVersion::Uninitialized {
            Ok(())
        } else {
            Err(ProgramError::UninitializedAccount)
        }
    }

    pub fn duration(&self) -> ApproximateSeconds {
        self.staked_until - self.staked_at
    }
}

impl ViewerStakePool {
    pub const LEN: usize = 97;
    pub fn uninitialized(&self) -> ProgramResult {
        if self.version == StateVersion::Uninitialized {
            Ok(())
        } else {
            Err(ProgramError::AccountAlreadyInitialized)
        }
    }
    /// Error if not initialized
    pub fn initialized(&self) -> ProgramResult {
        if self.version != StateVersion::Uninitialized {
            Ok(())
        } else {
            Err(ProgramError::UninitializedAccount)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::state::ViewerStake;

    use super::ViewerStakePool;
    use borsh::*;

    #[test]
    fn test() {
        let data = ViewerStakePool::default().try_to_vec().unwrap();
        assert_eq!(data.len(), ViewerStakePool::LEN);
        let data = ViewerStake::default().try_to_vec().unwrap();
        assert_eq!(data.len(), ViewerStake::LEN);
    }
}
