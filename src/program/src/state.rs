//! Program owned state

use std::ops::Mul;
use std::time::Duration;

use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::clock::UnixTimestamp;
use solana_program::pubkey::Pubkey;
use solana_program::{entrypoint::ProgramResult, program_error::ProgramError};

use crate::errors::Error;
use crate::sdk::types::*;
use crate::types::*;

/// state version
#[repr(C)]
#[derive(Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub enum StateVersion {
    /// new
    Uninitialized,
    /// version 1
    V1,
}

impl Default for StateVersion {
    fn default() -> Self {
        StateVersion::Uninitialized
    }
}

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
