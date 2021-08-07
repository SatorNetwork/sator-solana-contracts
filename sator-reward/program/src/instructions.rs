//! Program instruction state
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use sator_sdk::program::PubkeyPatterns;
use sator_sdk::types::{
    ApproximateSeconds, MintPubkey, SignerPubkey, TokenAccountPubkey, TokenAmount,
};
use solana_program::clock::UnixTimestamp;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;
use solana_program::{entrypoint::ProgramResult, program_error::ProgramError};
use solana_program::{system_program, sysvar};

use crate::program_id;
use crate::types::Winner;

#[derive(Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema, Clone)]
pub struct InitializeShowInput {
    pub reward_lock_time: ApproximateSeconds,
}

#[derive(Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema, Clone)]
pub struct InitializeViewerInput {
    pub user_wallet: Pubkey,
}

#[derive(Debug, BorshDeserialize, BorshSerialize, BorshSchema, Clone)]
pub enum Instruction {
    InitializeShow(InitializeShowInput),
    InitializeViewer(InitializeViewerInput),
    InitializeQuiz(InitializeQuizInput),
    Claim,
}

/// Creates [Instruction::InitializeShow] instruction which initializes `show` and shows' `token_account`
///
/// Accounts:
///  * `system_program`        - *program, implicit* to create accounts
///  * `sysvar_rent`           - *program, implicit* ensure that `token_account` and  `show` are rent exempt.
///  * `spl_token_program`     - *program, implicit* spl token program to initialize `token_account`.
///  * `owner`                 - *signer, payer* and owner of `show`.
///  * `show`                  - *mutable, signer* not initialized not created account for show data.
///  * `show_authority`        - *implicit* program derived account from `32 bytes show public key` based `program_id`.
///  * `token_account`         - *implicit, mutable, derived* not created program derived account to create `spl_token`  under `show_authority`.
///  * `mint`                  - used to initialize `token_account` for reference
#[allow(clippy::too_many_arguments)]
pub fn initialize_show(
    owner: &SignerPubkey,
    show: &SignerPubkey,
    mint: &MintPubkey,
    input: InitializeShowInput,
) -> Result<solana_program::instruction::Instruction, ProgramError> {
    let show_authority = Pubkey::find_program_address_for_pubkey(&show.pubkey(), &program_id());
    let token_account =
        Pubkey::create_with_seed(&show_authority.0, "Show::token_account", &spl_token::id())?;
    Ok(solana_program::instruction::Instruction::new_with_borsh(
        crate::id(),
        &Instruction::InitializeShow(input),
        vec![
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(*owner, true),
            AccountMeta::new(*show, true),
            AccountMeta::new_readonly(show_authority.0, false),
            AccountMeta::new(token_account, false),
            AccountMeta::new_readonly(*mint, false),
        ],
    ))
}

/// Creates [Instruction::InitializeViewer] instruction which proves the user passed QR code check, so that derived marker account created.
///
/// Accounts:
///  * `system_program`  - *program, implicit* to create accounts
///  * `sysvar_rent`     - *program, implicit* ensure that `token_account` and  `show` are rent exempt.
///  * `owner`           - *signer, payer*  owner of `show`
///  * `show`            - used to validate `owner`
///  * `show_authority` - *implicit* program derived account from `32 bytes show public key` based `program_id`.
///  * `viewer`         - *implicit, derived* from `show_authority` and `input.user_wallet`
#[allow(clippy::too_many_arguments)]
pub fn initialize_viewer(
    owner: &SignerPubkey,
    show: &Pubkey,
    input: InitializeViewerInput,
) -> Result<solana_program::instruction::Instruction, ProgramError> {
    let (show_authority_pubkey, _) =
        Pubkey::find_program_address_for_pubkey(&show.pubkey(), &program_id());
    let (viewer_pubkey, _) = Pubkey::create_with_seed_for_pubkey(
        &show_authority_pubkey,
        &input.user_wallet,
        &program_id(),
    )?;
    Ok(solana_program::instruction::Instruction::new_with_borsh(
        crate::id(),
        &Instruction::InitializeViewer(input),
        vec![
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
            AccountMeta::new_readonly(*owner, true),
            AccountMeta::new_readonly(*show, false),
            AccountMeta::new_readonly(show_authority_pubkey, false),
            AccountMeta::new(viewer_pubkey, false),
        ],
    ))
}

#[repr(C)]
#[derive(Debug, BorshDeserialize, BorshSerialize, BorshSchema, Default, Clone)]
pub struct InitializeQuizInput {
    pub winners: [Winner; 5],
}

/// Creates [Instruction::InitializeQuiz] instruction which initializes `quiz` with results. Validates winner is viewer.
/// `show`'s `quizes` latest number must be provided.
///
/// Accounts:
///  * `system_program`  - *program, implicit* to create accounts
///  * `sysvar_rent`     - *program, implicit* ensure that `quiz` are rent exempt.
///  * `clock`           - *program, implicit* to calculate prize won time
///  * `show`            - used to validate `owner` and `quiz` and tak
///  * `owner`           - *signer, payer* and owner of `show`.
///  * `show_authority` - *implicit* program derived account from `32 bytes show public key` based `program_id`.
///  * `quiz`            - *mutable, derived* from `show` + index
//   * `winners`         - *collection* to validate winners are viewers
#[allow(clippy::too_many_arguments)]
pub fn initialize_quiz(
    owner: &SignerPubkey,
    show: &Pubkey,
    quizes: u16,
    winners: Vec<Pubkey>,
    input: InitializeQuizInput,
) -> Result<solana_program::instruction::Instruction, ProgramError> {
    let (show_authority_pubkey, _) = Pubkey::find_program_address_for_pubkey(show, &program_id());
    let (quiz_pubkey, _) = Pubkey::create_with_seed_index(
        &show_authority_pubkey,
        "Show::quizes",
        quizes as u64,
        &program_id(),
    )?;
    let winners = winners
        .into_iter()
        .map(|x| AccountMeta::new_readonly(x, false))
        .collect();
    Ok(solana_program::instruction::Instruction::new_with_borsh(
        crate::id(),
        &Instruction::InitializeQuiz(input),
        [
            vec![
                AccountMeta::new_readonly(system_program::id(), false),
                AccountMeta::new_readonly(sysvar::rent::id(), false),
                AccountMeta::new_readonly(*owner, true),
                AccountMeta::new(*show, false),
                AccountMeta::new_readonly(show_authority_pubkey, false),
                AccountMeta::new(quiz_pubkey, false),
            ],
            winners,
        ]
        .concat(),
    ))
}

/// Creates [Instruction::Claim] wins on behalf of user. Transfers tokens from show account to user account, sets win claimed.
///
/// Accounts:
///  * `spl_token`         
///  * `show`               - used to validate `owner` and `quiz`
///  * `owner`              - *signer, payer* and owner of `show`.
///  * `show_authority`     - *implicit* program derived account from `32 bytes show public key` based `program_id`.
//   * `winner`             - will be find in each quiz
//   * `show_token_account` - *derived* source
//   * `user_token_account` - destination
///  * `quizes`             - *mutable, derived* to claim rewards from
#[allow(clippy::too_many_arguments)]
pub fn claim(
    owner: &SignerPubkey,
    show: &SignerPubkey,
    winner: Pubkey,
    user_token_account: TokenAccountPubkey,
    quizes: Vec<Pubkey>,
) -> Result<solana_program::instruction::Instruction, ProgramError> {
    let show_authority = Pubkey::find_program_address_for_pubkey(show, &program_id());
    let show_token_account =
        Pubkey::create_with_seed(&show_authority.0, "Show::token_account", &spl_token::id())?;
    let quizes = quizes
        .into_iter()
        .map(|x| AccountMeta::new(x, false))
        .collect();
    Ok(solana_program::instruction::Instruction::new_with_borsh(
        crate::id(),
        &Instruction::Claim,
        [
            vec![
                AccountMeta::new_readonly(system_program::id(), false),
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly(*show, false),
                AccountMeta::new_readonly(*owner, true),
                AccountMeta::new_readonly(show_authority.0, false),
                AccountMeta::new(winner, false),
                AccountMeta::new(show_token_account, false),
                AccountMeta::new(user_token_account, false),
            ],
            quizes,
        ]
        .concat(),
    ))
}
