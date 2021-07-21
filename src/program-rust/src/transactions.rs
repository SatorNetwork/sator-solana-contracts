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

use crate::{instruction::InitializeStakeInput, program_id, sdk::types::MintPubkey, state};

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
    transaction.sign(&[
        owner
        , &stake
        ], recent_blockhash);
    (transaction, stake.pubkey())
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

    transaction.sign(
        &[&payer, account],
        recent_blockhash,
    );

    transaction
}
