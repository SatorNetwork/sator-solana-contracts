use sator_stake_viewer::{
    instruction::{InitializeStakePoolInput, StakeInput},
    state::ViewerStakePool,
    types::Rank,
};
use solana_clap_utils::keypair::signer_from_path;
use solana_client::rpc_client::RpcClient;
use solana_program::{clock::UnixTimestamp, pubkey::Pubkey};
use solana_sdk::{
    commitment_config::CommitmentConfig,
    native_token::lamports_to_sol,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};

fn main() {
    let config = solana_cli_config::Config::default();
    let keypair =
        solana_sdk::signature::read_keypair_file("/home/dz/validator-keypair.json".to_string())
            .unwrap();
    let rpc_client = RpcClient::new("https://api.devnet.solana.com".to_string());
    solana_logger::setup_with_default("solana=debug");

    let rent = rpc_client
        .get_minimum_balance_for_rent_exemption(ViewerStakePool::LEN)
        .unwrap();

    let own_mint = true;
    let mut token_account = None;
    let mint = {
        if own_mint {
            let mint = Keypair::new();
            let transaction = sator_sdk_test::spl_transactions::create_initialize_mint(
                &keypair,
                &mint,
                &keypair.pubkey(),
                100000000,
                2,
                rpc_client.get_recent_blockhash().unwrap().0,
            );
            let signature = rpc_client
                .send_and_confirm_transaction_with_spinner_and_commitment(
                    &transaction,
                    CommitmentConfig::confirmed(),
                )
                .unwrap();

            println!("mint {:?}", signature);

            let (transaction, pubkey) = sator_sdk_test::spl_transactions::create_token_account(
                1000000000,
                &mint.pubkey(),
                &keypair,
                &keypair,
                rpc_client.get_recent_blockhash().unwrap().0,
            );

            token_account = Some(pubkey);
            let signature = rpc_client
                .send_and_confirm_transaction_with_spinner_and_commitment(
                    &transaction,
                    CommitmentConfig::confirmed(),
                )
                .unwrap();

            println!("user token account (wallet): {:?}", pubkey);

            let transaction = sator_sdk_test::spl_transactions::mint_to(
                &keypair,
                &mint.pubkey(),
                &pubkey,
                &keypair,
                1000000000,
                rpc_client.get_recent_blockhash().unwrap().0,
            );
            let signature = rpc_client
                .send_and_confirm_transaction_with_spinner_and_commitment(
                    &transaction,
                    CommitmentConfig::confirmed(),
                )
                .unwrap();
            mint.pubkey().to_string()
        } else {
            "13kBuVtxUT7CeddDgHfe61x3YdpBWTCKeB2Zg2LC4dab".to_string()
        }
    };

    let stake = Keypair::new();
    let mut transaction = Transaction::new_with_payer(
        &[sator_stake_viewer::instruction::initialize_stake_pool(
            &keypair.pubkey(),
            &keypair.pubkey(),
            &stake.pubkey(),
            &mint.parse().unwrap(),
            InitializeStakePoolInput {
                ranks: [
                    Rank {
                        minimal_staking_time: 0,
                        amount: 100,
                    },
                    Rank {
                        minimal_staking_time: 30 * 60,
                        amount: 200,
                    },
                    Rank {
                        minimal_staking_time: 60 * 60,
                        amount: 300,
                    },
                    Rank {
                        minimal_staking_time: 2 * 60 * 60,
                        amount: 500,
                    },
                ],
            },
        )
        .unwrap()],
        Some(&keypair.pubkey()),
    );

    let (recent_blockhash, fee_calculator) = rpc_client.get_recent_blockhash().unwrap();

    let signers = vec![&keypair, &stake];
    transaction.sign(&signers, recent_blockhash);

    let signature = rpc_client
        .send_and_confirm_transaction_with_spinner_and_commitment(
            &transaction,
            CommitmentConfig::confirmed(),
        )
        .unwrap();

    println!("initialize {:?}", signature);

    if let Some(token) = token_account {
        let mut transaction = Transaction::new_with_payer(
            &[sator_stake_viewer::instruction::stake(
                &keypair.pubkey(),
                &keypair.pubkey(),
                &stake.pubkey(),
                &token,
                StakeInput {
                    duration: 100500,
                    amount: 42,
                },
            )
            .unwrap()
            .0],
            Some(&keypair.pubkey()),
        );

        let (recent_blockhash, fee_calculator) = rpc_client.get_recent_blockhash().unwrap();

        let signers = vec![&keypair];
        transaction.sign(&signers, recent_blockhash);

        let signature = rpc_client
            .send_and_confirm_transaction_with_spinner_and_commitment(
                &transaction,
                CommitmentConfig::confirmed(),
            )
            .unwrap();

        println!("stake {:?}", signature);
    }
}
