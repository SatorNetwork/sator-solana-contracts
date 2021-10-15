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
    options.all_exists = false;
    let stake_pool_owner =  Keypair::new();

    let stake_pool_owner:Vec<_> = [0x7d, 0x36, 0x17, 0xd5, 0x2c, 0xc8, 0x64, 0xf2, 0x9a, 0x39, 0x2f, 0x8b, 0xb6, 0x40, 0x4e, 0xf9, 0xcd, 0x4c, 0x85, 0xa8, 0x9a, 0xbe, 0x3c, 0xfe, 0xa9, 0xe1, 0xad, 0xbc, 0xb5, 0x40, 0x2a, 0xf9, 0x1, 0xb6, 0x4b, 0x6f, 0x7e, 0x76, 0xc3, 0x3d, 0x4b, 0xf6, 0xcf, 0xc6, 0xb4, 0x6, 0xd8, 0x1f, 0xcf, 0x96, 0xe1, 0x67, 0x5a, 0xdf, 0xd3, 0x22, 0xbf, 0xe2, 0x8a, 0xa6, 0x92, 0xa, 0xee, 0x2f].iter().map(|x| *x as u8).collect();
    let stake_pool_owner = Keypair::from_bytes(&stake_pool_owner[0..64]).unwrap();

    if options.all_exists {
        println!("using existign stake");


        let mut fee_payer = "[115, 91, 202, 172, 215, 254, 239, 102, 127, 239, 39, 117, 165, 14, 239, 60, 242, 138, 216, 4, 183, 230, 36, 122, 133, 128, 12, 201, 176, 200, 144, 182, 17, 64, 8, 222, 37, 225, 40, 90, 140, 94, 207, 194, 215, 172, 41, 156, 184, 231, 78, 111, 144, 102, 2, 211, 156, 35, 90, 19, 91, 13, 43, 209]".to_string();
        let mut fee_payer = std::io::Cursor::new(fee_payer);
        let fee_payer = solana_sdk::signature::read_keypair(&mut fee_payer).unwrap();

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
                &fee_payer.pubkey(),
                &stake_pool_owner.pubkey(),
                &stakepool.pubkey(),
                &userWallett,
                StakeInput {
                    duration: 100500,
                    amount: 42,
                },
            )
            .unwrap()
            .0],
            Some(&fee_payer.pubkey(),),
        );

        let (recent_blockhash, fee_calculator) = rpc_client.get_recent_blockhash().unwrap();

        let signers = vec![&fee_payer, &stake_pool_owner];
        transaction.sign(&signers, recent_blockhash);

        let signature = rpc_client
            .send_and_confirm_transaction_with_spinner_and_commitment(
                &transaction,
                CommitmentConfig::confirmed(),
            )
            .unwrap();

        println!("stake {:?}", signature);

    } else {
        let fee_payer =
            solana_sdk::signature::read_keypair_file("/home/dz/validator-keypair.json".to_string())
                .unwrap();

        let fee_payer:Vec<_> = [0xc2, 0x30, 0x16, 0x0, 0x95, 0x1b, 0xf8, 0x86, 0xf8, 0x71, 0x31, 0xab, 0x7d, 0x9d, 0x3b, 0x9d, 0x74, 0x6, 0x8d, 0xa6, 0xe1, 0xf0, 0x3, 0xd7, 0xdb, 0x26, 0xca, 0x5d, 0x98, 0x32, 0x2e, 0x4b, 0x35, 0x4, 0x1, 0x3b, 0xf, 0xdc, 0xe0, 0x52, 0x7e, 0x1c, 0x1f, 0xfc, 0x96, 0x68, 0x5f, 0xdc, 0x1d, 0xdd, 0x26, 0x7, 0xbf, 0x33, 0x1b, 0x1b, 0x84, 0xef, 0xf8, 0xd4, 0xec, 0x7d, 0xb7, 0xa6].iter().map(|x| *x as u8).collect();
        let fee_payer = Keypair::from_bytes(&fee_payer[0..64]).unwrap();

        println!("fee payer pk: {:?}", fee_payer.to_bytes());
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
                    &fee_payer,
                    &mint,
                    &stake_pool_owner.pubkey(),
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

                dbg!("mint trx :{:?}", signature);

                let mint:Vec<_> = [0x2b, 0xe7, 0x8c, 0x5, 0xbd, 0x7f, 0x6f, 0x7a, 0xb4, 0xd6, 0x68, 0x7a, 0xfa, 0xf3, 0xd6, 0x14, 0x9c, 0xce, 0x9a, 0xff, 0x72, 0x6a, 0x9, 0x40, 0x52, 0x16, 0x54, 0xe7, 0xe5, 0x75, 0xe0, 0x15, 0xc1, 0xc7, 0x6b, 0x43, 0x40, 0xe9, 0xdf, 0xc3, 0x9, 0x8a, 0x4f, 0xbd, 0x30, 0x99, 0xc4, 0x5d, 0x64, 0xcd, 0x43, 0xf5, 0xdf, 0x82, 0xf4, 0xc6, 0x4b, 0x6c, 0x5, 0x1c, 0xdc, 0xbc, 0x45, 0xd].iter().map(|x| *x as u8).collect();
                let mint = Keypair::from_bytes(&mint[0..64]).unwrap();

                println!("mint pk: {:?}", mint.to_bytes());

                let (transaction, user_account) = sator_sdk_test::spl_transactions::create_token_account(
                    1000000000,
                    &mint.pubkey(),
                    &stake_pool_owner,
                    &fee_payer,
                    rpc_client.get_recent_blockhash().unwrap().0,
                );

                let user_account:Vec<_> = [0xa, 0x51, 0xfd, 0xbe, 0xde, 0x59, 0xb7, 0x1c, 0x2c, 0x9e, 0x56, 0x8a, 0xad, 0x9, 0x57, 0xc3, 0x19, 0x25, 0xfa, 0xca, 0x6f, 0x17, 0xe1, 0xec, 0x11, 0x5d, 0xd5, 0xad, 0x20, 0xd6, 0xe, 0xc2, 0x94, 0xeb, 0x96, 0xa, 0xe7, 0xd4, 0xec, 0x4c, 0x87, 0xb4, 0x34, 0x38, 0xd9, 0x73, 0xa6, 0x48, 0xaf, 0xbe, 0xa0, 0xf7, 0xa5, 0x52, 0x2e, 0x6b, 0xcf, 0x9f, 0xa7, 0xda, 0x78, 0x89, 0x9b, 0x10].iter().map(|x| *x as u8).collect();
                let user_account = Keypair::from_bytes(&user_account[0..64]).unwrap();

                println!("user token account pk: {:?}", user_account.to_bytes());
                let pubkey = user_account.pubkey();

                token_account = Some(pubkey);
                let signature = rpc_client
                    .send_and_confirm_transaction_with_spinner_and_commitment(
                        &transaction,
                        CommitmentConfig::confirmed(),
                    )
                    .unwrap();

                println!("user token account (wallet): {:?}", pubkey);

                let transaction = sator_sdk_test::spl_transactions::mint_to(
                    &fee_payer,
                    &mint.pubkey(),
                    &pubkey,
                    &stake_pool_owner,
                    10000000,
                    rpc_client.get_recent_blockhash().unwrap().0,
                );
                let signature = rpc_client
                    .send_and_confirm_transaction_with_spinner_and_commitment(
                        &transaction,
                        CommitmentConfig::confirmed(),
                    )
                    .unwrap();

                println!("minted to trx: {:?}", signature);

                mint.pubkey().to_string()
            } else {
                "13kBuVtxUT7CeddDgHfe61x3YdpBWTCKeB2Zg2LC4dab".to_string()
            }
        };

        let stake = Keypair::new();
        println!("stake pool pk: {:?}", stake.to_bytes());
        let mut transaction = Transaction::new_with_payer(
            &[sator_stake_viewer::instruction::initialize_stake_pool(
                &fee_payer.pubkey(),
                &stake_pool_owner.pubkey(),
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
            Some(&fee_payer.pubkey()),
        );

        let (recent_blockhash, fee_calculator) = rpc_client.get_recent_blockhash().unwrap();

        let signers = vec![&fee_payer, &stake, &stake_pool_owner];
        transaction.sign(&signers, recent_blockhash);

        let signature = rpc_client
            .send_and_confirm_transaction_with_spinner_and_commitment(
                &transaction,
                CommitmentConfig::confirmed(),
            )
            .unwrap();

        println!("initialize trx: {:?}", signature);

        if let Some(token) = token_account {
            let mut transaction = Transaction::new_with_payer(
                &[sator_stake_viewer::instruction::stake(
                    &fee_payer.pubkey(),
                    &stake_pool_owner.pubkey(),
                    &stake.pubkey(),
                    &token,
                    StakeInput {
                        duration: 100500,
                        amount: 42,
                    },
                )
                .unwrap()
                .0],
                Some(&fee_payer.pubkey()),
            );

            let (recent_blockhash, fee_calculator) = rpc_client.get_recent_blockhash().unwrap();

            let signers = vec![&fee_payer, &stake_pool_owner];
            transaction.sign(&signers, recent_blockhash);

            let signature = rpc_client
                .send_and_confirm_transaction_with_spinner_and_commitment(
                    &transaction,
                    CommitmentConfig::confirmed(),
                )
                .unwrap();

            println!("viewer stake trx: {:?}", signature);
        }
    }
}
