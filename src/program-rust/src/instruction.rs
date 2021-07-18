//! Program owned state

use std::time::Duration;

use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::clock::UnixTimestamp;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;
use solana_program::sysvar;
use solana_program::{entrypoint::ProgramResult, program_error::ProgramError};

use crate::sdk::program::PubkeyPatterns;
use crate::sdk::types::*;
use crate::{program_id, state, types::*};

#[derive(Debug, BorshDeserialize, BorshSerialize, BorshSchema)]
pub struct InitializeStakeInput {
    pub rank_requirements: RankRequirements,
    pub minimal_staking_time : ApproximateSeconds,
    pub mint_sao: MintPubkey,
}

#[derive(Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub struct LockInput {

}

#[derive(Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub struct UnlockInput {

}

#[derive(Debug,  BorshDeserialize, BorshSerialize, BorshSchema)]
pub enum Instruction 
{   
    /// Initilized stake account 
    InitializeStake(InitializeStakeInput),
    Lock(LockInput),
    UnlockLock(LockInput),
}

/// Creates [Instructions::InitializeStake] instruction
///  * `rent`            - *program, implicit* ensure that `token_account` and  `stake` are rent exempt
///  * `spl_token`       - *program, implicit* spl token program to initialize `token_account`
///  * `owner`           - *signer* owner of `stake`
///  * `stake`           - *mutable* not initialized not created account for stake data, 
///  *` stake_authority` - *implicit* program derived account from program + owner public key
///  *` token_account`   - *mutable, derived* not created program derived account to create `spl_token`  under `stake_authority`
#[allow(clippy::too_many_arguments)]
pub fn initialize_stake(        
    owner: &SignerPubkey,
    stake: &ProgramDerivedPubkey,    
    token_account: &TokenAccountPubKey,    
    input: InitializeStakeInput,
) -> Result<solana_program::instruction::Instruction, ProgramError> {
    let accounts = vec![
        AccountMeta::new_readonly(sysvar::rent::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(*owner, true),
        AccountMeta::new(*stake, false),
        AccountMeta::new_readonly(Pubkey::find_program_address_for_pubkey(owner, &program_id()).0, false),                
    ];
    Ok(solana_program::instruction::Instruction::new_with_borsh(
        crate::id(),
        &Instruction::InitializeStake(input),
        accounts,
    ))
}