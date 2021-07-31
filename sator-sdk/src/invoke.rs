//! Program state processor
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
};

use super::{
    program::{AccountPatterns, PubkeyPatterns},
    types::{ ProgramPubkey},
};

/// Creates system account externally signed
pub fn create_account<'a>(
    funder: AccountInfo<'a>,
    account_to_create: AccountInfo<'a>,
    required_lamports: u64,
    space: u64,
    owner: &ProgramPubkey,
    _system_program: &AccountInfo<'a>,
) -> ProgramResult {
    invoke(
        &system_instruction::create_account(
            &funder.key,
            &account_to_create.key,
            required_lamports,
            space,
            owner,
        ),
        &[funder.clone(), account_to_create.clone()],
    )
}

pub fn create_account_signed<'a>(
    funder: AccountInfo<'a>,
    account_to_create: AccountInfo<'a>,
    required_lamports: u64,
    space: u64,
    owner: &ProgramPubkey,
    signers_seeds: &[&[&[u8]]],
) -> ProgramResult {
    invoke_signed(
        &system_instruction::create_account(
            &funder.key,
            &account_to_create.key,
            required_lamports,
            space,
            owner,
        ),
        &[funder.clone(), account_to_create.clone()],
        signers_seeds,
    )
}

pub struct ProgramPubkeySignature {
    bytes: [u8; 32],
    bump_seed: [u8; 1],
}

impl ProgramPubkeySignature {
    pub fn new(account: &AccountInfo, bump_seed: u8) -> Self {
        Self {
            bytes: account.pubkey().to_bytes(),
            bump_seed: [bump_seed],
        }
    }

    // [&[&[u8]]; 1]
    // [&[u8], 2]
    pub fn signature(&self) -> [&[u8]; 2] {
        [&self.bytes[..32], &self.bump_seed]
    }
}

/// Create account
#[allow(clippy::too_many_arguments)]
pub fn create_account_with_seed_signed<'a>(
    _system_program: &AccountInfo<'a>, // explicit parameters usage just to indicate dependency
    from_account: &AccountInfo<'a>,
    to_account: &AccountInfo<'a>,
    base: &AccountInfo<'a>,
    seed: String,
    lamports: u64,
    space: u64,
    program_owner: &ProgramPubkey,    
    signers_seeds: &ProgramPubkeySignature,
) -> ProgramResult {
    let instruction = &system_instruction::create_account_with_seed(
        from_account.key,
        to_account.key,
        base.key,
        seed.as_str(),
        lamports,
        space,
        &program_owner.pubkey(),
    );

    solana_program::program::invoke_signed(
        instruction,
        &[from_account.clone(), to_account.clone(), base.clone()],
        &[&signers_seeds.signature()[..]],
    )
}


/// Initialize mint
pub fn initialize_mint<'a>(
    mint_to_initialize: AccountInfo<'a>,
    mint_authority: AccountInfo<'a>,
    decimals: u8,
) -> ProgramResult {
    invoke(
        &spl_token::instruction::initialize_mint(
            &spl_token::id(),
            &mint_to_initialize.key,
            mint_authority.key,
            None,
            decimals,
        )?,
        &[mint_to_initialize, mint_authority],
    )
}

/// Initialize mint
pub fn initialize_mint_signed<'a>(
    mint: &AccountInfo<'a>,
    pool: &Pubkey,
    owner_authority: &AccountInfo<'a>,
    decimals: u8,
    rent_account: &AccountInfo<'a>,
    bump_seed: u8,
) -> ProgramResult {
    let authority_signature = [&pool.to_bytes()[..32], b"Mint".as_ref(), &[bump_seed]];
    let authority_signature = &[&authority_signature[..]];

    let instruction = &spl_token::instruction::initialize_mint(
        &spl_token::id(),
        &mint.key,
        owner_authority.key,
        None,
        decimals,
    )?;

    invoke_signed(
        instruction,
        &[mint.clone(), rent_account.clone()],
        authority_signature,
    )
}

/// transfer with on chain authority
pub fn spl_token_transfer_signed<'a>(
    _spl_token_program: &AccountInfo<'a>,
    source: &AccountInfo<'a>,
    destination: &AccountInfo<'a>,
    owner_authority: &AccountInfo<'a>,
    amount: u64,
    signature: &ProgramPubkeySignature,
) -> Result<(), ProgramError> {
    let tx = spl_token::instruction::transfer(
        &spl_token::id(),
        source.key,
        destination.key,
        owner_authority.key,
        &[&owner_authority.key],
        amount,
    )?;
    invoke_signed(
        &tx,
        &[source.clone(), destination.clone(), owner_authority.clone()],
        &[&signature.signature()[..]],
    )
}

/// Transfer tokens with user transfer authority
pub fn spl_token_transfer<'a>(
    _spl_token_program: &AccountInfo<'a>,
    source: &AccountInfo<'a>,
    destination: &AccountInfo<'a>,
    authority: &AccountInfo<'a>,
    amount: u64,
) -> Result<(), ProgramError> {
    let tx = spl_token::instruction::transfer(
        &spl_token::id(),
        source.key,
        destination.key,
        authority.key,
        &[&authority.pubkey()],
        amount,
    )?;
    invoke(
        &tx,
        &[source.clone(), destination.clone(), authority.clone()],
    )
}

/// Issue a spl_token `MintTo` instruction
pub fn token_mint_to<'a>(
    pool: &Pubkey,
    mint: AccountInfo<'a>,
    destination: AccountInfo<'a>,
    authority: AccountInfo<'a>,
    bump_seed: u8,
    amount: u64,
) -> Result<(), ProgramError> {
    let authority_signature_seeds = [&pool.to_bytes()[..32], &[bump_seed]];
    let signers = &[&authority_signature_seeds[..]];
    let ix = spl_token::instruction::mint_to(
        &spl_token::id(),
        mint.key,
        destination.key,
        authority.key,
        &[],
        amount,
    )?;

    invoke_signed(&ix, &[mint, destination, authority], signers)
}

/// Issue a spl_token `Burn` instruction
pub fn burn_tokens<'a>(
    pool: &Pubkey,
    burn_account: AccountInfo<'a>,
    mint: AccountInfo<'a>,
    authority: AccountInfo<'a>,
    bump_seed: u8,
    amount: u64,
) -> Result<(), ProgramError> {
    let authority_signature_seeds = [&pool.to_bytes()[..32], &[bump_seed]];
    let signers = &[&authority_signature_seeds[..]];
    let ix = spl_token::instruction::burn(
        &spl_token::id(),
        burn_account.key,
        mint.key,
        authority.key,
        &[],
        amount,
    )?;

    invoke_signed(&ix, &[burn_account, mint, authority], signers)
}

/// Burn tokens with user authority
pub fn burn_tokens_with_user_authority<'a>(
    burn_account: AccountInfo<'a>,
    mint: AccountInfo<'a>,
    authority: AccountInfo<'a>,
    amount: u64,
) -> Result<(), ProgramError> {
    let tx = spl_token::instruction::burn(
        &spl_token::id(),
        burn_account.key,
        mint.key,
        authority.key,
        &[],
        amount,
    )?;

    invoke(&tx, &[burn_account, mint, authority])
}

/// in program invoke to create program signed seeded account
#[allow(clippy::too_many_arguments)]
pub fn create_derived_account<'a>(
    payer: AccountInfo<'a>,
    account_to_create: AccountInfo<'a>,
    base: AccountInfo<'a>,
    seed: &str,
    required_lamports: u64,
    space: u64,
    owner: &Pubkey,
    signer_seeds: &[&[u8]],
) -> ProgramResult {
    solana_program::program::invoke_signed(
        &system_instruction::create_account_with_seed(
            &payer.key,
            &account_to_create.key,
            &base.key,
            seed,
            required_lamports,
            space,
            owner,
        ),
        &[payer.clone(), account_to_create.clone(), base.clone()],
        &[&signer_seeds],
    )
}

/// Initialize
pub fn initialize_token_account_signed<'a>(
    token_account: &AccountInfo<'a>,
    mint: &AccountInfo<'a>,
    owner: &AccountInfo<'a>,
    rent_account: &AccountInfo<'a>,
    signers_seeds: &ProgramPubkeySignature,
) -> ProgramResult {
    let instruction = &spl_token::instruction::initialize_account(
        &spl_token::id(),
        &token_account.pubkey(),
        &mint.pubkey(),
        &owner.pubkey(),
    )?;

    invoke_signed(
        instruction,
        &[
            token_account.clone(),
            mint.clone(),
            rent_account.clone(),
            owner.clone(),
        ],
        &[&signers_seeds.signature()[..]],
    )
}
