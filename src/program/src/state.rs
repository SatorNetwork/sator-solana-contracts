//! Program owned state

use std::ops::Mul;
use std::time::Duration;

use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::clock::{UnixTimestamp};
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

/// pool state and rules
#[repr(C)]
#[derive(Debug, BorshDeserialize, BorshSerialize, BorshSchema, Default)]
pub struct ViewerStakePool {
    pub version: StateVersion,
    pub ranks: [Rank; 4],
    // can initialize state and change rules
    pub owner: SignerPubkey,
}
/// stake
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

    pub fn duration (&self) -> ApproximateSeconds {
        self.staked_until - self.staked_at
    }

}

impl ViewerStakePool {
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

    pub fn calculate_reward(&self, account: ViewerStake) -> Result<u64, ProgramError> {

        let mut multiplier = Rank::ONE;
        for r in self.ranks.iter() {
            if account.amount >= r.amount && account.staked_until - account.staked_at >= r.minimal_staking_time  {
                multiplier = Rank::ONE + r.multiplier as u128;
            }
        }
        let multiplier = fixed::types::U114F14::from_num(multiplier) / Rank::ONE;
        let amount = fixed::types::U114F14::from_num( account.amount);
        multiplier.checked_mul(amount).map(|x| x.to_num()).ok_or_else(||Error::UnstakeOverflow.into())   
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
