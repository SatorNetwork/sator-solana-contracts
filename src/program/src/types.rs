use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::clock::UnixTimestamp;
use solana_program::pubkey::Pubkey;
use solana_program::{entrypoint::ProgramResult, program_error::ProgramError};

use crate::sdk::types::*;

#[derive(Debug, BorshDeserialize, BorshSerialize, BorshSchema, Default, Clone, Copy)]
pub struct Rank {
    pub minimal_staking_time: ApproximateSeconds,
    /// 5000 is 1.5x multiplier. so we do not start from 10000, but from addition on top of 1X to avoid negative rewards. fixed point with 4 fractional positions.
    pub multiplier: BasisPointsMultiplier,
    /// amount of token required to reach this rank
    pub amount: TokenAmount,
}

impl Rank {
    pub const ONE:u128 = 10_000;
}
