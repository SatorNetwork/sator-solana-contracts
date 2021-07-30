use crate::{
    instruction::StakeInput,    
    spl_transactions,
    state::{ViewerStake, ViewerStakePool},
    tests_helpers::*,
    transactions::{self, warp_seconds},
};
use borsh::BorshDeserialize;
use sator_sdk::program::PubkeyPatterns;
use solana_program::native_token::sol_to_lamports;
use solana_program_test::*;
use solana_sdk::{    
    account::Account,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
    program::*,
};
use std::mem;

use crate::{
    instruction::InitializeStakePoolInput, processor::process_instruction, stake_viewer_program_id,
    spl_transactions::create_initialize_mint, state, transactions::initialize_stake, types::Rank,
};

pub fn new_program_test() -> ProgramTest {
    let mut program_test = ProgramTest::new(
        "sator_stake_viewer",
        stake_viewer_program_id(),
        processor!(process_instruction),
    );
    program_test.add_program("spl_token", spl_token::id(), None);
    program_test
}

#[tokio::test]
async fn flow() {
    let mut program_test = new_program_test();

    let stake_pool_owner = Keypair::new();
    let user_wallet = Keypair::new();

    program_test.add_account(
        stake_pool_owner.pubkey(),
        Account {
            lamports: u64::MAX / 32,
            ..<_>::default()
        },
    );

    program_test.add_account(
        user_wallet.pubkey(),
        Account {
            lamports: u64::MAX / 32,
            ..<_>::default()
        },
    );

    let mint = Keypair::new();
    let mut client = program_test.start_with_context().await;

    let transaction = create_initialize_mint(
        &stake_pool_owner,
        &mint,
        &stake_pool_owner.pubkey(),
        sol_to_lamports(10.),
        2,
        client.last_blockhash,
    );

    client
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    let minute = 60;
    let hour = 60 * minute;
    let amount = 1000;

    let (transaction, stake_pool) = initialize_stake(
        &stake_pool_owner,
        &mint.pubkey(),
        InitializeStakePoolInput {
            ranks: [
                Rank {
                    minimal_staking_time: 0,
                    amount,
                },
                Rank {
                    minimal_staking_time: 1 * hour,
                    amount: amount * 2,
                },
                Rank {
                    minimal_staking_time: 2 * hour,
                    amount: amount * 3,
                },
                Rank {
                    minimal_staking_time: 3 * hour,
                    amount: amount * 4,
                },
            ],
        },
        client.last_blockhash,
    );

    client
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    let stake_authority =
        Pubkey::find_program_address_for_pubkey(&stake_pool.pubkey(), &crate::stake_viewer_program_id());
    let token_account = Pubkey::create_with_seed(
        &stake_authority.0,
        "ViewerStakePool::token_account",
        &spl_token::id(),
    )
    .unwrap();

    let token_account_state =
        get_token_account_state(&mut client.banks_client, &token_account).await;
    assert_eq!(token_account_state.mint, mint.pubkey());
    let stake_state: ViewerStakePool = client
        .banks_client
        .get_account_data_with_borsh(stake_pool.pubkey())
        .await
        .unwrap();

    assert!(stake_state.ranks[3].minimal_staking_time > 0);
    assert!(stake_state.ranks[3].amount > 0);

    let transaction = spl_transactions::mint_to(
        &stake_pool_owner,
        &mint.pubkey(),
        &token_account,
        &stake_pool_owner,
        10000000000,
        client.last_blockhash,
    );

    client
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    let (transaction, user_token_account) = spl_transactions::create_token_account(
        10000000,
        &mint.pubkey(),
        &user_wallet,
        &user_wallet,
        client.last_blockhash,
    );

    client
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    let transaction = spl_transactions::mint_to(
        &stake_pool_owner,
        &mint.pubkey(),
        &token_account,
        &stake_pool_owner,
        10000000000,
        client.last_blockhash,
    );

    client
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    let transaction = spl_transactions::mint_to(
        &stake_pool_owner,
        &mint.pubkey(),
        &user_token_account,
        &stake_pool_owner,
        1000000,
        client.last_blockhash,
    );
    client
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    let stake_duration = hour;
    let (transaction, stake_account) = transactions::stake(
        &user_wallet,
        &stake_pool.pubkey(),
        &user_token_account,
        StakeInput {
            amount: 1000,
            duration: stake_duration,
        },
        client.last_blockhash,
    );

    client
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    let user_token_account_state =
        get_token_account_state(&mut client.banks_client, &user_token_account).await;
    let token_account_state =
        get_token_account_state(&mut client.banks_client, &token_account).await;

    let viewer_stake_account: ViewerStake = client
        .banks_client
        .get_account_data_with_borsh(stake_account.pubkey())
        .await
        .unwrap();
    assert_eq!(
        viewer_stake_account.staked_until - viewer_stake_account.staked_at,
        stake_duration
    );
    assert_eq!(viewer_stake_account.amount, 1000);

    let transaction = transactions::unstake(
        &user_wallet,
        &stake_pool.pubkey(),
        &user_token_account,
        &stake_pool_owner,
        client.last_blockhash,
    );

    client
        .banks_client
        .process_transaction(transaction)
        .await
        .expect_err("must fail to unlock");

    let (transaction, _) = transactions::stake(
        &user_wallet,
        &stake_pool.pubkey(),
        &user_token_account,
        StakeInput {
            amount: 2000,
            duration: stake_duration,
        },
        client.last_blockhash,
    );

    client
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    let viewer_stake_account: ViewerStake = client
        .banks_client
        .get_account_data_with_borsh(stake_account.pubkey())
        .await
        .unwrap();

    assert_eq!(
        viewer_stake_account.staked_until - viewer_stake_account.staked_at,
        stake_duration
    );
    assert_eq!(viewer_stake_account.amount, 3000);

    warp_seconds(&mut client, 5 * hour).await;

    let transaction = transactions::unstake(
        &user_wallet,
        &stake_pool.pubkey(),
        &user_token_account,
        &stake_pool_owner,
        client.last_blockhash,
    );

    client
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    client
        .banks_client
        .get_account_data_with_borsh::<ViewerStake>(stake_account.pubkey())
        .await
        .expect_err("account was burned");
}
