use borsh::BorshDeserialize;
use sator_sdk::program::*;
use sator_sdk::types::*;
use solana_program::{clock::Clock, system_instruction};
use solana_program_test::*;
use solana_sdk::{
    account::Account,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use std::mem;

use crate::instruction::InitializeQuizInput;
use crate::instruction::InitializeShowInput;
use crate::instruction::InitializeViewerInput;

pub fn initialize_show(
    owner: &Keypair,
    mint: &MintPubkey,
    input: InitializeShowInput,
    recent_blockhash: solana_program::hash::Hash,
) -> (Transaction, Pubkey) {
    let show = Keypair::new();
    let mut transaction = Transaction::new_with_payer(
        &[
            crate::instruction::initialize_show(&owner.pubkey(), &show.pubkey(), mint, input)
                .expect("could create derived keys"),
        ],
        Some(&owner.pubkey()),
    );
    transaction.sign(&[owner, &show], recent_blockhash);
    (transaction, show.pubkey())
}

pub fn initialize_viewer(
    owner: &Keypair,
    show: &Pubkey,
    input: InitializeViewerInput,
    recent_blockhash: solana_program::hash::Hash,
) -> Transaction {
    let mut transaction = Transaction::new_with_payer(
        &[
            crate::instruction::initialize_viewer(&owner.pubkey(), &show.pubkey(), input)
                .expect("could create derived keys"),
        ],
        Some(&owner.pubkey()),
    );
    transaction.sign(&[owner], recent_blockhash);
    transaction
}

pub fn initialize_quiz(
    owner: &Keypair,
    show: &Pubkey,
    index: u16,
    input: InitializeQuizInput,
    winners: Vec<Pubkey>,
    recent_blockhash: solana_program::hash::Hash,
) -> Transaction {
    let mut transaction = Transaction::new_with_payer(
        &[crate::instruction::initialize_quiz(
            &owner.pubkey(),
            &show.pubkey(),
            index,
            winners,
            input,
        )
        .expect("could create derived keys")],
        Some(&owner.pubkey()),
    );
    transaction.sign(&[owner], recent_blockhash);
    transaction
}

pub fn claim(
    owner: &Keypair,
    show: &Pubkey,
    winner: &Pubkey,
    user_token_account: &TokenAccountPubkey,
    quizes: Vec<Pubkey>,
    recent_blockhash: solana_program::hash::Hash,
) -> Transaction {
    let mut transaction = Transaction::new_with_payer(
        &[crate::instruction::claim(
            &owner.pubkey(),
            &show.pubkey(),
            winner,
            user_token_account,
            quizes,
        )
        .expect("could create derived keys")],
        Some(&owner.pubkey()),
    );
    transaction.sign(&[owner], recent_blockhash);
    transaction
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

pub async fn warp_seconds(program_context: &mut ProgramTestContext, seconds: ApproximateSeconds) {
    let ticks_per_slot = program_context.genesis_config().ticks_per_slot();
    assert_eq!(ticks_per_slot, 64);
    assert!(
        seconds as u64 > 10 * ticks_per_slot,
        "clocks are very approximate"
    );

    let before = get_clock(program_context).await.unix_timestamp;
    loop {
        warp(program_context, 100).await;
        let after = get_clock(program_context).await.unix_timestamp;
        if after > before + seconds {
            break;
        }
    }
}

pub async fn warp(program_context: &mut ProgramTestContext, slots: u64) {
    let slot = program_context.banks_client.get_root_slot().await.unwrap();
    program_context.warp_to_slot(slot + slots).unwrap();
}

pub async fn get_clock(program_context: &mut ProgramTestContext) -> Clock {
    let clock = program_context
        .banks_client
        .get_account(solana_program::sysvar::clock::id())
        .await
        .unwrap()
        .unwrap();
    let clock: Clock = bincode::deserialize(&clock.data[..]).unwrap();
    clock
}
