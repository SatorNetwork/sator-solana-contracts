use sator_stake_viewer::{
    instruction::InitializeStakePoolInput, state::ViewerStakePool, types::Rank,
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

    let stake = Keypair::new();
    let mut transaction = Transaction::new_with_payer(
        &[sator_stake_viewer::instruction::initialize_stake_pool(
            &keypair.pubkey(),
            &stake.pubkey(),
            &"13kBuVtxUT7CeddDgHfe61x3YdpBWTCKeB2Zg2LC4dab"
                .parse()
                .unwrap(),
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

    println!("{:?}", signature);
}
