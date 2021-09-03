use sator_reward::{instruction::InitializeShowInput, state::Show};
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
        .get_minimum_balance_for_rent_exemption(Show::LEN)
        .unwrap();

    let show = Keypair::new();
    let mut transaction = Transaction::new_with_payer(
        &[sator_reward::instruction::initialize_show(
            &keypair.pubkey(),
            &show.pubkey(),
            &"13kBuVtxUT7CeddDgHfe61x3YdpBWTCKeB2Zg2LC4dab"
                .parse()
                .unwrap(),
            InitializeShowInput {
                reward_lock_time: 1 * 60 * 60,
            },
        )
        .unwrap()],
        Some(&keypair.pubkey()),
    );

    let (recent_blockhash, fee_calculator) = rpc_client.get_recent_blockhash().unwrap();

    let signers = vec![&keypair, &show];
    transaction.sign(&signers, recent_blockhash);

    let signature = rpc_client
        .send_and_confirm_transaction_with_spinner_and_commitment(
            &transaction,
            CommitmentConfig::confirmed(),
        )
        .unwrap();

    println!("{:?}", signature);
}
