use borsh::BorshDeserialize;
use solana_program::{program_pack::Pack, system_instruction};
use solana_sdk::{
    account::Account,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use std::mem;

use sator_sdk::{
    program::PubkeyPatterns,
    types::{Lamports, MintPubkey, TokenAccountPubkey},
};

pub fn create_token_account(
    account_rent: Lamports,
    mint: &Pubkey,
    owner: &Keypair,
    payer: &Keypair,
    recent_blockhash: solana_program::hash::Hash,
) -> (Transaction, Keypair) {
    let token_account = Keypair::new();
    let instructions = vec![
        system_instruction::create_account(
            &payer.pubkey(),
            &token_account.pubkey(),
            account_rent,
            spl_token::state::Account::LEN as u64,
            &spl_token::id(),
        ),
        spl_token::instruction::initialize_account(
            &spl_token::id(),
            &token_account.pubkey(),
            mint,
            &owner.pubkey(),
        )
        .expect("spl initialization parameters are right"),
    ];

    let mut transaction = Transaction::new_with_payer(&instructions, Some(&payer.pubkey()));

    transaction.sign(&[&token_account, payer], recent_blockhash);

    (transaction, token_account)
}

/// Simplified mint instruction
pub fn create_initialize_mint(
    payer: &Keypair,
    mint: &Keypair,
    authority: &Pubkey,
    account_rent: u64,
    decimals: u8,
    recent_blockhash: solana_program::hash::Hash,
) -> Transaction {
    let mut transaction = Transaction::new_with_payer(
        &[
            system_instruction::create_account(
                &payer.pubkey(),
                &mint.pubkey(),
                account_rent,
                spl_token::state::Mint::LEN as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_mint(
                &spl_token::id(),
                &mint.pubkey(),
                authority,
                None,
                decimals,
            )
            .unwrap(),
        ],
        Some(&payer.pubkey()),
    );

    transaction.sign(&[payer, mint], recent_blockhash);

    transaction
}
pub fn mint_to(
    payer: &Keypair,
    mint: &MintPubkey,
    token_account: &TokenAccountPubkey,
    owner: &Keypair,
    amount: u64,
    recent_blockhash: solana_program::hash::Hash,
) -> Transaction {
    let instruction = spl_token::instruction::mint_to(
        &spl_token::id(),
        &mint.pubkey(),
        &token_account.pubkey(),
        &owner.pubkey(),
        &[],
        amount,
    )
    .unwrap();
    let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));

    transaction.sign(&[payer, owner], recent_blockhash);
    transaction
}
