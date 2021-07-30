use borsh::BorshDeserialize;
use sator_sdk::types::*;
use sator_sdk::program::*;
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


use crate::{
    instruction::{InitializeStakePoolInput, StakeInput},
    stake_viewer_program_id,    
    state,
};

pub fn initialize_stake(
    owner: &Keypair,
    mint: &MintPubkey,
    input: InitializeStakePoolInput,
    recent_blockhash: solana_program::hash::Hash,
) -> (Transaction, Pubkey) {
    let stake = Keypair::new();
    let mut transaction =
        Transaction::new_with_payer(
            &[crate::instruction::initialize_stake_pool(
                &owner.pubkey(),
                &stake.pubkey(),
                mint,
                input,
            )
            .expect("could create derived keys")],
            Some(&owner.pubkey()),
        );
    transaction.sign(&[owner, &stake], recent_blockhash);
    (transaction, stake.pubkey())
}

pub fn stake(
    user_wallet: &Keypair,
    stake_pool: &Pubkey,
    token_account_source: &TokenAccountPubkey,
    input: StakeInput,
    recent_blockhash: solana_program::hash::Hash,
) -> (Transaction, Pubkey) {
    let (instruction, stake) = crate::instruction::stake(
        &user_wallet.pubkey(),
        stake_pool,
        token_account_source,
        input,
    )
    .expect("could create derived keys");
    let mut transaction = Transaction::new_with_payer(&[instruction], Some(&user_wallet.pubkey()));
    transaction.sign(&[user_wallet], recent_blockhash);
    (transaction, stake)
}

pub fn unstake(
    wallet: &Keypair,
    stake_pool: &Pubkey,
    token_account_target: &TokenAccountPubkey,
    stake_pool_owner: &Keypair,
    recent_blockhash: solana_program::hash::Hash,
) -> Transaction {
    let instruction = crate::instruction::unstake(
        &wallet.pubkey(),
        stake_pool,
        token_account_target,
        &stake_pool_owner.pubkey(),
    )
    .expect("could create derived keys");
    let mut transaction =
        Transaction::new_with_payer(&[instruction], Some(&stake_pool_owner.pubkey()));
    transaction.sign(&[stake_pool_owner], recent_blockhash);
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
