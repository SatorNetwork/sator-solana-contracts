use crate::{
    instruction::StakeInput,
    state::{ViewerStake, ViewerStakePool},
    tests_helpers::*,
    transactions::{self, warp_seconds},
};
use sator_sdk::program::PubkeyPatterns;
use solana_program::native_token::sol_to_lamports;
use solana_program_test::*;
use solana_sdk::{
    account::Account,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};

use sator_sdk_test::spl_transactions;

use crate::{
    instruction::InitializeStakePoolInput, processor::process_instruction, stake_viewer_program_id,
    state, transactions::initialize_stake_pool, types::Rank,
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

/// see dbg! writes for full flow tracing
#[tokio::test]
async fn flow() {
    let mut program_test = new_program_test();

    let stake_pool_owner = Keypair::new();
    let fee_payer = Keypair::new();
    let user_wallet = Keypair::new();

    program_test.add_account(
        fee_payer.pubkey(),
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

    dbg!("Create staking mint");
    let transaction = spl_transactions::create_initialize_mint(
        &fee_payer,
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

    dbg!("Initializing stake pool");
    let (transaction, stake_pool) = initialize_stake_pool(
        &fee_payer,
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

    let stake_authority = Pubkey::find_program_address_for_pubkey(
        &stake_pool.pubkey(),
        &crate::stake_viewer_program_id(),
    );
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

    dbg!("Minting to stake pool wallet");
    let transaction = spl_transactions::mint_to(
        &fee_payer,
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

    dbg!("Create user token account wallet");
    let (transaction, user_token_account) = spl_transactions::create_token_account(
        10000000,
        &mint.pubkey(),
        &stake_pool_owner,
        &fee_payer,
        client.last_blockhash,
    );

    client
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    dbg!("Minting to user token account wallet");
    let transaction = spl_transactions::mint_to(
        &fee_payer,
        &mint.pubkey(),
        &user_token_account.pubkey(),
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
    dbg!("Staking from user wallet");
    let (transaction, stake_account) = transactions::stake(
        &fee_payer,
        &stake_pool_owner,
        &stake_pool.pubkey(),
        &user_token_account.pubkey(),
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
        get_token_account_state(&mut client.banks_client, &user_token_account.pubkey()).await;
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

    dbg!("Unstaking from lock not yet possible");
    let transaction = transactions::unstake(
        &fee_payer,
        &stake_pool.pubkey(),
        &user_token_account.pubkey(),
        &stake_pool_owner,
        client.last_blockhash,
    );

    client
        .banks_client
        .process_transaction(transaction)
        .await
        .expect_err("must fail to unlock");

    dbg!("Staking more on existing stake");
    let (transaction, _) = transactions::stake(
        &fee_payer,
        &stake_pool_owner,
        &stake_pool.pubkey(),
        &user_token_account.pubkey(),
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

    dbg!("Unstaking from lock with success");
    let user_token_account_state_before =
        get_token_account_state(&mut client.banks_client, &user_token_account.pubkey()).await;

    let transaction = transactions::unstake(
        &fee_payer,
        &stake_pool.pubkey(),
        &user_token_account.pubkey(),
        &stake_pool_owner,
        client.last_blockhash,
    );

    client
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    let user_token_account_state_after =
        get_token_account_state(&mut client.banks_client, &user_token_account.pubkey()).await;
    let unstaked = user_token_account_state_after.amount - user_token_account_state_before.amount;
    assert_eq!(unstaked, 3000);

    client
        .banks_client
        .get_account_data_with_borsh::<ViewerStake>(stake_account.pubkey())
        .await
        .expect_err("account was burned");
}
