use crate::tests_helpers::*;
use crate::{
    instruction::InitializeShowInput,
    instruction::{InitializeQuizInput, InitializeViewerInput, WinnerInput},
    program_id,
    state::{Quiz, Show, Viewer},
    tests_helpers::*,
    transactions::{self, initialize_quiz, initialize_show, initialize_viewer, warp_seconds},
};
use borsh::BorshDeserialize;
use sator_sdk::program::PubkeyPatterns;
use solana_program::native_token::sol_to_lamports;
use solana_program_test::*;
use solana_sdk::{
    account::Account,
    instruction::{AccountMeta, Instruction},
    program::*,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use std::mem;

use sator_sdk_test::spl_transactions;

use crate::processor::process_instruction;

pub fn new_program_test() -> ProgramTest {
    let mut program_test = ProgramTest::new(
        "sator_reward",
        program_id(),
        processor!(process_instruction),
    );
    program_test.add_program("spl_token", spl_token::id(), None);
    program_test
}

#[tokio::test]
async fn flow() {
    let mut program_test = new_program_test();

    let show_owner = Keypair::new();
    let user_wallet = Keypair::new();

    program_test.add_account(
        show_owner.pubkey(),
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

    let transaction = spl_transactions::create_initialize_mint(
        &show_owner,
        &mint,
        &show_owner.pubkey(),
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

    let (transaction, show) = initialize_show(
        &show_owner,
        &mint.pubkey(),
        InitializeShowInput {
            reward_lock_time: 2 * hour,
        },
        client.last_blockhash,
    );

    client
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    let show_state = client
        .banks_client
        .get_account_data_with_borsh::<Show>(show.pubkey())
        .await
        .unwrap();

    assert_eq!(show_state.owner, show_owner.pubkey());
    assert_eq!(show_state.lock_time, 2 * hour);

    let (show_authority_pubkey, _) =
        Pubkey::find_program_address_for_pubkey(&show.pubkey(), &program_id());

    let show_authority = Pubkey::find_program_address_for_pubkey(&show.pubkey(), &program_id());
    let token_account =
        Pubkey::create_with_seed(&show_authority.0, Show::token_account, &spl_token::id()).unwrap();

    let transaction = spl_transactions::mint_to(
        &show_owner,
        &mint.pubkey(),
        &token_account,
        &show_owner,
        1000,
        client.last_blockhash,
    );

    client
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    let transaction = initialize_viewer(
        &show_owner,
        &show.pubkey(),
        InitializeViewerInput {
            user_wallet: user_wallet.pubkey(),
        },
        client.last_blockhash,
    );

    client
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    let show_state = client
        .banks_client
        .get_account_data_with_borsh::<Show>(show.pubkey())
        .await
        .unwrap();

    assert_eq!(show_state.owner, show_owner.pubkey());
    assert_eq!(show_state.lock_time, 2 * hour);

    let (viewer_pubkey, _) = Pubkey::create_with_seed_for_pubkey(
        &show_authority_pubkey,
        &user_wallet.pubkey(),
        &program_id(),
    )
    .unwrap();

    let transaction = initialize_quiz(
        &show_owner,
        &show.pubkey(),
        0,
        InitializeQuizInput {
            winners: vec![WinnerInput {
                points: 42,
                owner: user_wallet.pubkey(),
            }],
        },
        vec![viewer_pubkey],
        client.last_blockhash,
    );

    client
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    warp_seconds(&mut client, 3 * hour).await;

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

    let (quiz_pubkey, _) =
        Pubkey::create_with_seed_index(&show_authority_pubkey, "Show::quizes", 0, &program_id())
            .unwrap();

    let transaction = transactions::claim(
        &show_owner,
        &show.pubkey(),
        &user_wallet.pubkey(),
        &user_token_account,
        vec![quiz_pubkey],
        client.last_blockhash,
    );

    client
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    let account = get_token_account_state(&mut client.banks_client, &user_token_account).await;
    assert_eq!(account.amount, 42);
}
