use std::io::{Read, Cursor};
use sator_sdk::program::PubkeyPatterns;
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

#[derive(Debug, Default)]
pub struct Options {
    pub mint: bool,
    pub all_exists: bool,
}

fn main() {
    let config = solana_cli_config::Config::default();
    let mut options: Options = Options::default();
    options.mint = true;
    //options.all_exists = true;


    if options.all_exists {
        let mut key = "[115, 91, 202, 172, 215, 254, 239, 102, 127, 239, 39, 117, 165, 14, 239, 60, 242, 138, 216, 4, 183, 230, 36, 122, 133, 128, 12, 201, 176, 200, 144, 182, 17, 64, 8, 222, 37, 225, 40, 90, 140, 94, 207, 194, 215, 172, 41, 156, 184, 231, 78, 111, 144, 102, 2, 211, 156, 35, 90, 19, 91, 13, 43, 209]".to_string();
        let mut cursor = std::io::Cursor::new(key);
        let keypair = solana_sdk::signature::read_keypair(&mut cursor).unwrap();        
        
        let rpc_client = RpcClient::new("https://api.devnet.solana.com".to_string());
        solana_logger::setup_with_default("solana=debug");

        let stakepool: Pubkey = "CpMhtwfw4yMuL8Nk83zri4HyLnrtc19xXRzvzv6G4vRw".parse().unwrap();
        let feee: Pubkey = "2ALZgMNre2qynTTyxWtgWG6L2L56n39aBGegS1yvxwya".parse().unwrap();
        let stakeAuth: Pubkey = "7jq2SaaZEsCgaxYmmjiZqXJtdxwRNoRaWFLSTZPX2e5L".parse().unwrap();
        let userWallett: Pubkey = "3juXiCWJwusUqDgN9T5oVjic5h4CmbueyHZ7B1oBWZEX".parse().unwrap();
        let tokenAccountStakeTargett: Pubkey = "9sKWcvyXfuubEwgxnNGHUzsndCpoPRJzGL1KavPU29Vq".parse().unwrap();
        let stakeAccountt: Pubkey = "7jq2SaaZEsCgaxYmmjiZqXJtdxwRNoRaWFLSTZPX2e5L".parse().unwrap();       
         
        let mut transaction = Transaction::new_with_payer(
            &[sator_stake_viewer::instruction::stake(
                &keypair.pubkey(),
                &keypair.pubkey(),
                &stakepool.pubkey(),
                &userWallett,
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

    } else {
        let keypair =
            solana_sdk::signature::read_keypair_file("/home/dz/validator-keypair.json".to_string())
                .unwrap();
        let rpc_client = RpcClient::new("https://api.devnet.solana.com".to_string());
        solana_logger::setup_with_default("solana=debug");

        let rent = rpc_client
            .get_minimum_balance_for_rent_exemption(ViewerStakePool::LEN)
            .unwrap();

        let mut token_account = None;
        let mint = {
            if options.mint {
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
                    1,
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
}
