use borsh::BorshDeserialize;
use solana_program_test::*;
use solana_sdk::{
    account::Account,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use std::mem;

use crate::{instruction::InitializeStakeInput, processor::process_instruction, program_id, spl_transactions::create_initialize_mint, state, transactions::initialize_stake, types::RankRequirements};

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

    let transaction = create_initialize_mint(&owner,&mint, &owner.pubkey(), 1000000, 2, client.last_blockhash);

    client
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();
        
        
    let (transaction, stake) = initialize_stake(
        &owner,
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
            mint: mint.pubkey(),
        },
        client.last_blockhash,
    );
    
    client
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();
}
