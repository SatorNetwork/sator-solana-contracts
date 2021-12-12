//! Program state processor
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program::{invoke,}, 
};
use spl_token::instruction::initialize_account;

/// Initialize token account
pub fn initialize_token_account<'a>(
    account_to_initialize: AccountInfo<'a>,
    mint: AccountInfo<'a>,
    owner: AccountInfo<'a>,
) -> ProgramResult {
    invoke(
        &initialize_account(
            &spl_token::id(),
            &account_to_initialize.key,
            mint.key,
            owner.key,
        )?,
        &[account_to_initialize, mint, owner],
    )
}
