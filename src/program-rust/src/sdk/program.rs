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

    fn create_with_seed_for_pubkey(
        base: &Pubkey,
        seed: &Pubkey,
        owner: &ProgramPubkey,
    ) -> Result<(ProgramDerivedPubkey, String), PubkeyError>;

    /// Generate certain program address
    fn find_program_address_from_2_keys<'a>(
        a: &Pubkey,
        b: &Pubkey,
        program_id: &ProgramPubkey,
    ) -> (ProgramDerivedPubkey, u8);
}

impl PubkeyPatterns for Pubkey {
    fn find_program_address_for_pubkey(
        seed: &Pubkey,
        program_id: &ProgramPubkey,
    ) -> (ProgramDerivedPubkey, u8) {
        Pubkey::find_program_address(&[&seed.to_bytes()[..32]], program_id)
    }

    fn create_with_seed_for_pubkey(
        base: &Pubkey,
        seed: &Pubkey,
        owner: &ProgramPubkey,
    ) -> Result<(ProgramDerivedPubkey, String), PubkeyError> {
        // Pubkey.to_string is longer than 32 chars limit in Solana for seed
        // ETH compatible something
        let seed = seed.to_bytes();
        let seed = bs58::encode(&seed[..20]).into_string();
        let pubkey = Pubkey::create_with_seed(base, &seed, owner)?;
        Ok((pubkey, seed))
    }

    fn find_program_address_from_2_keys<'a>(
        a: &Pubkey,
        b: &Pubkey,
        program_id: &ProgramPubkey,
    ) -> (ProgramDerivedPubkey, u8) {
        Pubkey::find_program_address(&[&a.to_bytes()[..32], &b.to_bytes()[..32]], program_id)
    }
}
