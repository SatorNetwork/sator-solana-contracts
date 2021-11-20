use std::io::{Read, Cursor};
use solana_clap_utils::keypair::signer_from_path;
use solana_cli_config::Config;
use solana_client::rpc_client::RpcClient;
use solana_program::{clock::UnixTimestamp, pubkey::Pubkey};
use solana_sdk::{
    commitment_config::CommitmentConfig,
    native_token::lamports_to_sol,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};
use std::fs;
use metaplex_token_metadata;
use metaplex_token_metadata::{
    id, instruction,
    state::{Creator, Data, PREFIX},
};

fn main() {
    let config: Config = solana_cli_config::load_config_file("/sator-cli/src/config.devnet.yaml").unwrap();

    let mut fee_payer = fs::read(config.keypair_path).unwrap();
    let mut fee_payer = std::io::Cursor::new(fee_payer);
    let fee_payer = solana_sdk::signature::read_keypair(&mut fee_payer).unwrap();

    let rpc_client = RpcClient::new("https://api.devnet.solana.com".to_string());
    solana_logger::setup_with_default("solana=debug");

    let mint_pubkey: Pubkey = "EuRVbM38Dvseeei6Be5q8SP31d7dKySsRAM5vV48DrBs".parse().unwrap();
    let ref id = id();
    let metadata_seeds = &[PREFIX.as_bytes(), &id.to_bytes(), mint_pubkey.as_ref()];

    let (metadata_account, _) = Pubkey::find_program_address(metadata_seeds, id);
    dbg!("metadata_account: {:}",metadata_account);
    let instruction = metaplex_token_metadata::instruction::create_metadata_accounts(
        metaplex_token_metadata::id(),
        metadata_account,
        mint_pubkey,
        fee_payer.pubkey(),
        fee_payer.pubkey(),
        fee_payer.pubkey(),
        "Sator".to_string(),
        "SAO".to_string(),
        "https://raw.githubusercontent.com/SatorNetwork/sator-solana-contracts/main/assets/sao.json".to_string(),
        None,
        // vec![
        //     Creator { address: fee_payer.pubkey(), verified: true, share: 100 },
        // ],
        10_000,
        true,
        true,
    );
    let mut transaction = Transaction::new_with_payer(
        &[instruction],
        Some(&fee_payer.pubkey(),),
    );

    let (recent_blockhash, fee_calculator) = rpc_client.get_recent_blockhash().unwrap();

    let signers = vec![&fee_payer, ];
    transaction.sign(&signers, recent_blockhash);

    let signature = rpc_client
        .send_and_confirm_transaction_with_spinner_and_commitment(
            &transaction,
            CommitmentConfig::confirmed(),
        )
        .unwrap();

    println!("signature {:?}", signature);
}
