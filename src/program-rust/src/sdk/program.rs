//! In program helpers

use std::mem;

use borsh::BorshSerialize;
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::{Pubkey, PubkeyError},
    system_instruction,
};

use super::types::*;

/// some well know often users patters for program derived keys
pub trait PubkeyPatterns {
    /// Find authority address and bump seed for `seed` key    
    fn find_program_address_for_pubkey(
        seed: &Pubkey,
        program_id: &ProgramPubkey,
    ) -> (ProgramDerivedPubkey, u8);
}

impl PubkeyPatterns for Pubkey {
    fn find_program_address_for_pubkey(seed: &Pubkey, program_id: &ProgramPubkey) -> (ProgramDerivedPubkey, u8) {
        Pubkey::find_program_address(&[&seed.to_bytes()[..32]], program_id)
    }
}