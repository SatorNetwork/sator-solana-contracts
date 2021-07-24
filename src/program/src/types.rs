use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::clock::UnixTimestamp;
use solana_program::pubkey::Pubkey;
use solana_program::{entrypoint::ProgramResult, program_error::ProgramError};

use crate::sdk::types::*;

#[derive(Debug, BorshDeserialize, BorshSerialize, BorshSchema, Default, Clone, Copy)]
pub struct Ranks {
    pub minimal_staking_time: ApproximateSeconds,
    /// 5000 is 1.5x multiplier.
    pub multiplier: BasisPointsMultiplier,
    /// amount of token required to reach this rank
    pub amount: TokenAmount,
}
