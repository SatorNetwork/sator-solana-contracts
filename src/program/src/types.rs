use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::clock::UnixTimestamp;
use solana_program::pubkey::Pubkey;
use solana_program::{entrypoint::ProgramResult, program_error::ProgramError};

use crate::sdk::types::*;

#[derive(Debug, BorshDeserialize, BorshSerialize, BorshSchema, Default, Clone, Copy)]
pub struct Rank {
    pub minimal_staking_time: ApproximateSeconds,

    /// amount of token required to reach this rank
    pub amount: TokenAmount,
}

impl Rank {
    pub const ONE: u128 = 10_000;
}
