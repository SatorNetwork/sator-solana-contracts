use borsh::BorshDeserialize;
use solana_program::system_instruction;
use solana_program_test::*;
use solana_sdk::{
    account::Account,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use std::mem;

use crate::{instruction::{InitializeStakeInput, LockInput}, program_id, sdk::types::{MintPubkey, TokenAccountPubkey}, state};

pub fn initialize_stake(
    owner: &Keypair,
    mint: &MintPubkey,
    input: InitializeStakeInput,
    recent_blockhash: solana_program::hash::Hash,
) -> (Transaction, Pubkey) {
    let stake = Keypair::new();
    let mut transaction = Transaction::new_with_payer(
        &[
            crate::instruction::initialize_stake(&owner.pubkey(), &stake.pubkey(), mint, input)
                .expect("could create derived keys"),
        ],
        Some(&owner.pubkey()),
    );
    transaction.sign(&[owner, &stake], recent_blockhash);
    (transaction, stake.pubkey())
}


pub fn lock(
    wallet: &Keypair,
    stake: &Pubkey, 
    token_account_source: &TokenAccountPubkey,   
    input: LockInput,
    recent_blockhash: solana_program::hash::Hash,
) -> (Transaction, Pubkey) {
    let (instruction, lock) = crate::instruction::lock(&wallet.pubkey(), stake, token_account_source, input)
    .expect("could create derived keys");
    let mut transaction = Transaction::new_with_payer(
        &[
            instruction
        ],
        Some(&wallet.pubkey()),
    );
    transaction.sign(&[wallet], recent_blockhash);
    (transaction, lock)
}


pub fn create_system_account(
    payer: &Keypair,
    account: &Keypair,
    rent: u64,
    space: u64,
    program_id: &Pubkey,
    recent_blockhash: solana_program::hash::Hash,
) -> Transaction {
    let mut transaction = Transaction::new_with_payer(
        &[system_instruction::create_account(
            &payer.pubkey(),
            &account.pubkey(),
            rent,
            space,
            program_id,
        )],
        Some(&payer.pubkey()),
    );

    transaction.sign(&[&payer, account], recent_blockhash);

    transaction
}
