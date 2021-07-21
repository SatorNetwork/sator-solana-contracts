use borsh::BorshDeserialize;
use solana_program::{program_pack::Pack, system_instruction};
use solana_program_test::*;
use solana_sdk::{
    account::Account,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use std::mem;

use crate::{instruction::InitializeStakeInput, program_id, state};

pub fn create_token_account(
    account_rent: u64,
    mint: &Pubkey,
    owner: &Keypair,
    payer: &Keypair,
    recent_blockhash: solana_program::hash::Hash,
) -> (Transaction, Pubkey) {
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

    transaction.sign(&[owner, &token_account, payer], recent_blockhash);

    (transaction, token_account.pubkey())
}
