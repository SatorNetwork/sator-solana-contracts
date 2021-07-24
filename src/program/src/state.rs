//! Program owned state

use std::time::Duration;

use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::clock::{UnixTimestamp};
use solana_program::pubkey::Pubkey;
use solana_program::{entrypoint::ProgramResult, program_error::ProgramError};

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

/// pool state and rules
#[repr(C)]
#[derive(Debug, BorshDeserialize, BorshSerialize, BorshSchema, Default)]
pub struct ViewerStake {
    pub version: StateVersion,
    pub ranks: [Ranks; 4],
    // can initialize state and change rules
    pub owner: SignerPubkey,
}
/// lock
#[repr(C)]
#[derive(Debug, BorshDeserialize, BorshSerialize, BorshSchema, Default)]
pub struct ViewerLock {
    pub version: StateVersion,
    pub locked_at: UnixTimestamp,
    pub locked_until: UnixTimestamp,
    /// user owner of lock
    pub owner: SignerPubkey,
    pub amount: TokenAmount,
}

impl ViewerLock {
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
}

impl ViewerStake {
    pub const LEN: usize = 113;
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
    use crate::state::ViewerLock;

    use super::ViewerStake;
    use borsh::*;

    #[test]
    fn test() {
        let data = ViewerStake::default().try_to_vec().unwrap();
        assert_eq!(data.len(), ViewerStake::LEN);
        let data = ViewerLock::default().try_to_vec().unwrap();
        assert_eq!(data.len(), ViewerLock::LEN);
    }
}
