use crate::{sdk::program::PubkeyPatterns, state::ViewerStake, test_helpers::*};
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
    spl_transactions::create_initialize_mint, state, transactions::initialize_stake,
    types::RankRequirements,
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
            lamports: 1000000000000,
            ..<_>::default()
        },
    );

    program_test.add_account(
        user.pubkey(),
        Account {
            lamports: 1000000000000,
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

    let (transaction, stake) = initialize_stake(
        &owner,
        &mint.pubkey(),
        InitializeStakeInput {
            rank_requirements: [
                RankRequirements {
                    minimal_staking_time: 0,
                    multiplier: 1_0000,
                },
                RankRequirements {
                    minimal_staking_time: 2000,
                    multiplier: 2 * 1_0000,
                },
                RankRequirements {
                    minimal_staking_time: 2 * 2000,
                    multiplier: 3 * 1_0000,
                },
                RankRequirements {
                    minimal_staking_time: 3 * 2000,
                    multiplier: 4 * 1_0000,
                },
                RankRequirements {
                    minimal_staking_time: 4 * 2000,
                    multiplier: 5 * 1_0000,
                },
            ],
            minimal_staking_time: 1_000,
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

    let token_account = get_token_account_state(&mut client.banks_client, &token_account).await;
    assert_eq!(token_account.mint, mint.pubkey());
    let stake: ViewerStake = client
        .banks_client
        .get_account_data_with_borsh(stake.pubkey())
        .await
        .unwrap();

    assert!(stake.minimal_staking_time > 0);
    assert!(stake.rank_requirements[4].multiplier > 0);
}
