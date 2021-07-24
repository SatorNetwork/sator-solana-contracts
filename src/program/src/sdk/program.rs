//! In program helpers

use std::mem;

use borsh::{BorshDeserialize, BorshSerialize};
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
    fn find_program_address_for_pubkeys<'a>(
        a: &Pubkey,
        b: &Pubkey,
        program_id: &ProgramPubkey,
    ) -> (ProgramDerivedPubkey, u8);

    /// pubkey
    fn pubkey(&self) -> Pubkey;
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

    fn find_program_address_for_pubkeys<'a>(
        a: &Pubkey,
        b: &Pubkey,
        program_id: &ProgramPubkey,
    ) -> (ProgramDerivedPubkey, u8) {
        Pubkey::find_program_address(&[&a.to_bytes()[..32], &b.to_bytes()[..32]], program_id)
    }

    fn pubkey(&self) -> Pubkey {
        *self
    }
}

pub trait AccountPatterns {
    /// validate key is equal to other key which assumed to  be derived
    fn is_derived<'b, K: Into<&'b ProgramPubkey>>(
        &self,
        owner: &Pubkey,
        program_id: K,
    ) -> Result<u8, ProgramError>;
    /// public key
    fn pubkey(&self) -> Pubkey;

    /// checks if program_id owner of self
    fn is_owner(&self, program_id: &ProgramPubkey) -> ProgramResult;

    /// checks if account is signer
    fn is_signer(&self) -> ProgramResult;

    fn deserialize<T: BorshDeserialize>(&self) -> Result<T, std::io::Error>;
}

impl<'a> AccountPatterns for AccountInfo<'a> {
    fn is_derived<'b, K: Into<&'b ProgramPubkey>>(
        &self,
        owner: &Pubkey,
        program_id: K,
    ) -> Result<u8, ProgramError> {
        let (expected_key, seed) =
            Pubkey::find_program_address_for_pubkey(owner, &program_id.into());

        if *self.key == expected_key {
            Ok(seed)
        } else {
            Err(ProgramError::InvalidSeeds)
        }
    }

    fn pubkey(&self) -> Pubkey {
        *self.key
    }

    fn is_owner(&self, program_id: &ProgramPubkey) -> ProgramResult {
        if self.owner != program_id {
            return Err(ProgramError::IncorrectProgramId);
        }

        Ok(())
    }

    fn is_signer(&self) -> ProgramResult {
        if !self.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        Ok(())
    }

    fn deserialize<T: BorshDeserialize>(&self) -> Result<T, std::io::Error> {
        let data = self
            .try_borrow_data()
            .expect("program is written with proper single borrow");
        T::try_from_slice(&data)
    }
}

/// errors if relation is not expected
pub fn is_derived(relation: Pubkey, related: &AccountInfo) -> ProgramResult {
    if relation != related.pubkey() {
        return Err(ProgramError::InvalidSeeds);
    }

    Ok(())
}


/// burns account
pub fn burn_account(burned: &AccountInfo, beneficiary: &AccountInfo) {
    let mut from = burned.try_borrow_mut_lamports().unwrap();
    let mut to = beneficiary.try_borrow_mut_lamports().unwrap();
    **to += **from;
    **from = 0;
}