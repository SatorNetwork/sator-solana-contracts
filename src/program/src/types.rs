use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::clock::UnixTimestamp;
use solana_program::pubkey::Pubkey;
use solana_program::{entrypoint::ProgramResult, program_error::ProgramError};

use crate::sdk::types::*;

#[derive(Debug, BorshDeserialize, BorshSerialize, BorshSchema, Default, Clone, Copy)]
pub struct RankRequirements {    
    pub minimal_staking_time: ApproximateSeconds,
    /// Must be more than 10000. Example, 20000 is 2X multiplier
    pub multiplier: BasisPointsMultiplier,
}
