use crate::{instruction::LockInput, sdk::program::PubkeyPatterns, spl_transactions, state::{ViewerLock, ViewerStake}, tests_helpers::*, transactions::{self, warp_seconds}};
use borsh::BorshDeserialize;
use solana_program::native_token::sol_to_lamports;
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
    instruction::InitializeStakeInput, processor::process_instruction, program_id,
    spl_transactions::create_initialize_mint, state, transactions::initialize_stake, types::Ranks,
};

pub fn new_program_test() -> ProgramTest {
    let mut program_test = ProgramTest::new(
        "sator_stake_viewer",
        program_id(),
        processor!(process_instruction),
    );
    program_test.add_program("spl_token", spl_token::id(), None);
    program_test
}

#[tokio::test]
async fn flow() {
    let mut program_test = new_program_test();

    let owner = Keypair::new();
    let user = Keypair::new();

    program_test.add_account(
        owner.pubkey(),
        Account {
            lamports: u64::MAX / 32,
            ..<_>::default()
        },
    );

    program_test.add_account(
        user.pubkey(),
        Account {
            lamports: u64::MAX / 32,
            ..<_>::default()
        },
    );

    let mint = Keypair::new();
    let mut client = program_test.start_with_context().await;

    let transaction = create_initialize_mint(
        &owner,
        &mint,
        &owner.pubkey(),
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
    let multiplier_one = 1_0000;
    let amount = 1000;

    let (transaction, stake) = initialize_stake(
        &owner,
        &mint.pubkey(),
        InitializeStakeInput {
            ranks: [
                Ranks {
                    minimal_staking_time: 0,
                    multiplier: multiplier_one,
                    amount,
                },
                Ranks {
                    minimal_staking_time: 1 * hour,
                    multiplier: 2 * multiplier_one,
                    amount: amount * 2,
                },
                Ranks {
                    minimal_staking_time: 2 * hour,
                    multiplier: 3 * multiplier_one,
                    amount: amount * 3,
                },
                Ranks {
                    minimal_staking_time: 3 * hour,
                    multiplier: 4 * multiplier_one,
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
        Pubkey::find_program_address_for_pubkey(&stake.pubkey(), &crate::program_id());
    let token_account = Pubkey::create_with_seed(
        &stake_authority.0,
        "ViewerStake::token_account",
        &spl_token::id(),
    )
    .unwrap();

    let token_account_state =
        get_token_account_state(&mut client.banks_client, &token_account).await;
    assert_eq!(token_account_state.mint, mint.pubkey());
    let stake_state: ViewerStake = client
        .banks_client
        .get_account_data_with_borsh(stake.pubkey())
        .await
        .unwrap();

    assert!(stake_state.ranks[3].minimal_staking_time > 0);
    assert!(stake_state.ranks[3].multiplier > 0);
    assert!(stake_state.ranks[3].amount > 0);

    let transaction = spl_transactions::mint_to(
        &owner,
        &mint.pubkey(),
        &token_account,
        &owner,
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
        &user,
        &user,
        client.last_blockhash,
    );

    client
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    let transaction = spl_transactions::mint_to(
        &owner,
        &mint.pubkey(),
        &token_account,
        &owner,
        10000000000,
        client.last_blockhash,
    );

    client
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    let transaction = spl_transactions::mint_to(
        &owner,
        &mint.pubkey(),
        &user_token_account,
        &owner,
        1000000,
        client.last_blockhash,
    );
    client
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    let lock_duration = hour;
    let (transaction, lock) = transactions::lock(
        &user,
        &stake.pubkey(),
        &user_token_account,
        LockInput {
            amount: 1000,
            duration: lock_duration,
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

    let viewer_lock: ViewerLock = client
        .banks_client
        .get_account_data_with_borsh(lock.pubkey())
        .await
        .unwrap();
    assert_eq!(
        viewer_lock.locked_until - viewer_lock.locked_at,
        lock_duration
    );
    assert_eq!(viewer_lock.amount, 1000);

    let transaction = transactions::unlock(
        &user,
        &stake.pubkey(),
        &user_token_account,
        client.last_blockhash,
    );

    client
        .banks_client
        .process_transaction(transaction)
        .await
        .expect_err("must fail to unlock");

        let (transaction, lock) = transactions::lock(
            &user,
            &stake.pubkey(),
            &user_token_account,
            LockInput {
                amount: 2000,
                duration: lock_duration,
            },
            client.last_blockhash,
        );
    
        client
            .banks_client
            .process_transaction(transaction)
            .await
            .unwrap();        

            
            let viewer_lock: ViewerLock = client
        .banks_client
        .get_account_data_with_borsh(lock.pubkey())
        .await
        .unwrap();


            assert_eq!(
                viewer_lock.locked_until - viewer_lock.locked_at,
                lock_duration
            );
            assert_eq!(viewer_lock.amount, 3000);    
            
        warp_seconds(&mut client, 5 * hour).await;

        let transaction = transactions::unlock(
            &user,
            &stake.pubkey(),
            &user_token_account,
            client.last_blockhash,
        );
    
        client
            .banks_client
            .process_transaction(transaction)
            .await
            .unwrap();        

        client
            .banks_client
            .get_account_data_with_borsh::<ViewerLock>(lock.pubkey())
            .await
            .expect_err("account was burned");
    

}
