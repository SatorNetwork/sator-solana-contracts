use solana_program::{program_pack::Pack, pubkey::Pubkey};
use solana_program_test::{BanksClient, ProgramTestContext};
use solana_sdk::account::Account;

use crate::sdk::program::PubkeyPatterns;

pub async fn get_token_account_state(
    client: &mut BanksClient,
    token: &Pubkey,
) -> spl_token::state::Account {
    let data = get_account(client, &token.pubkey()).await;
    spl_token::state::Account::unpack_from_slice(&data.data[..]).unwrap()
}

pub async fn get_account(client: &mut BanksClient, pubkey: &Pubkey) -> Account {
    client
        .get_account(*pubkey)
        .await
        .expect("account not found")
        .expect("account empty")
}
