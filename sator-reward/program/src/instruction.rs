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
    /// something that uniquely represent user
    pub user: Pubkey,
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
///  * `show_token_account`    - *implicit, mutable, derived* not created program derived account to create `spl_token`  under `show_authority`.
///  * `mint`                  - used to initialize `token_account` for reference
#[allow(clippy::too_many_arguments)]
pub fn initialize_show(
    owner: &SignerPubkey,
    show: &SignerPubkey,
    mint: &MintPubkey,
    input: InitializeShowInput,
) -> Result<solana_program::instruction::Instruction, ProgramError> {
    let show_authority = Pubkey::find_program_address_for_pubkey(&show.pubkey(), &program_id());
    let token_account_show =
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
            AccountMeta::new(token_account_show, false),
            AccountMeta::new_readonly(*mint, false),
        ],
    ))
}

/// Creates [Instruction::InitializeViewer] instruction which proves the user passed some check, so that derived marker account created.
/// Make sure user can participate in quizzes.
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
    let (viewer_pubkey, _) =
        Pubkey::create_with_seed_for_pubkey(&show_authority_pubkey, &input.user, &program_id())?;
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

#[derive(Debug, BorshDeserialize, BorshSerialize, BorshSchema, Default, Clone, Copy)]
pub struct WinnerInput {
    // user
    pub owner: Pubkey,
    pub points: u32,
}

#[repr(C)]
#[derive(Debug, BorshDeserialize, BorshSerialize, BorshSchema, Default, Clone)]
pub struct InitializeQuizInput {
    /// less than or equal to 5
    pub winners: Vec<WinnerInput>,
    /// amount of tokens to distribute for this quiz
    pub amount: TokenAmount,
}

/// Creates [Instruction::InitializeQuiz] instruction which initializes `quiz` with results. Validates winner is viewer.
/// `show`'s `quizzes` latest number must be provided.
/// Winners and viewers must be in same corresponding order (zip should work), and less or equal to 5.
///
/// Instruction does not forces specified locked amount to be presented on on `Show::token_account` which is risk for user will not be payed.
/// Accounts:
///  * `system_program`  - *program, implicit* to create accounts
///  * `sysvar_rent`     - *program, implicit* ensure that `quiz` are rent exempt.
///  * `sysvar_clock`    - *program, implicit* to calculate prize won time
///  * `owner`           - *signer, payer* and owner of `show`.
///  * `show`            - used to validate `owner` and `quiz` and tak
///  * `show_authority` - *implicit* program derived account from `32 bytes show public key` based `program_id`.
///  * `quiz`            - *mutable, derived* from `show` + 'ShowState::index`
//   * `viewers`         - *collection* to validate winners are viewers
#[allow(clippy::too_many_arguments)]
pub fn initialize_quiz(
    owner: &SignerPubkey,
    show: &Pubkey,
    show_quizzes_index: u16,
    winners: Vec<Pubkey>,
    input: InitializeQuizInput,
) -> Result<solana_program::instruction::Instruction, ProgramError> {
    let (show_authority_pubkey, _) = Pubkey::find_program_address_for_pubkey(show, &program_id());

    let (quiz_pubkey, _) = Pubkey::create_with_seed_index(
        &show_authority_pubkey,
        "Show::quizes",
        show_quizzes_index as u64,
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
                AccountMeta::new_readonly(sysvar::clock::id(), false),
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
///  * `spl_token`          -
///  * `owner`              - *signer, payer* and owner of `show`.
///  * `show`               - used to validate `owner` and `quiz`
///  * `show_authority`     - *implicit* program derived account from `32 bytes show public key` based `program_id`.
//   * `winner`             - user who got win points and stored in quiz data, will be find in each quiz
//   * `show_token_account` - *derived* source of tokens to transfer to `winner`
//   * `user_token_account` - destination
///  * `quizzes`            - *mutable, derived* to claim rewards from
#[allow(clippy::too_many_arguments)]
pub fn claim(
    owner: &SignerPubkey,
    show: &SignerPubkey,
    user_wallet_winner: &Pubkey,
    user_token_account: &TokenAccountPubkey,
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
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly(*owner, true),
                AccountMeta::new_readonly(*show, false),
                AccountMeta::new_readonly(show_authority.0, false),
                AccountMeta::new(*user_wallet_winner, false),
                AccountMeta::new(show_token_account, false),
                AccountMeta::new(*user_token_account, false),
            ],
            quizes,
        ]
        .concat(),
    ))
}
