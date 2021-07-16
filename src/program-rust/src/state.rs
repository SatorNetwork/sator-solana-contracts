//! Program owned state

use std::time::Duration;

use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::clock::UnixTimestamp;
use solana_program::pubkey::Pubkey;
use solana_program::{entrypoint::ProgramResult, program_error::ProgramError};

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

/// related to [UnixTimestamp]
type AppoximateSeconds = i64;

/// must be more than 10000
// fixed point with one = 1.0000
type BasisPointsMultiplier = u32;


#[derive(Debug, BorshDeserialize, BorshSerialize, BorshSchema, Default)]
pub struct RankRequirements {
    pub minimal_staking_time: AppoximateSeconds,
    pub multiplifer: BasisPointsMultiplier, 
}

type MintPubkey = Pubkey;
type TokenAccountPubKey = Pubkey;
type SignerPubkey = Pubkey;

/// pool state and rules
#[repr(C)]
#[derive(Debug, BorshDeserialize, BorshSerialize, BorshSchema, Default)]
pub struct ViewerSatorStake {
    /// version
    pub minimal_staking_time: UnixTimestamp,
    pub rank_requirements : [RankRequirements; 5],
    pub mint_sao : MintPubkey,   
    // can initialize state and change rules 
    pub owner : SignerPubkey,   
}


/// lock
#[repr(C)]
#[derive(Debug, BorshDeserialize, BorshSerialize, BorshSchema, Default)]
pub struct ViewerSatorLock {
    /// when the amount to be considered locked
    pub locked_at: UnixTimestamp,
    /// program derived token account used to store locked amount
    pub token_account: TokenAccountPubKey,
    /// user owner of lock
    pub owner: SignerPubkey,
}