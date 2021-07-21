

use borsh::BorshDeserialize;
use solana_program_test::*;
use solana_sdk::{account::Account, instruction::{AccountMeta, Instruction}, pubkey::Pubkey, signature::{Keypair, Signer}, transaction::Transaction};
use std::mem;

use crate::{instruction::InitializeStakeInput, program_id, state};

pub fn initialize_stake(owner: &Keypair, input: InitializeStakeInput, recent_blockhash: solana_program::hash::Hash) -> (Transaction, Pubkey) {
    let stake = Keypair::new();
    let mut transaction = Transaction::new_with_payer(
    &[
        crate::instruction::initialize_stake(
            &owner.pubkey(),
            &stake.pubkey(),
            input
        ).expect("could create derived keys")
    ],
    Some(&owner.pubkey())
    );
    transaction.sign(&[owner, &stake], recent_blockhash);
    (transaction, stake.pubkey())
}
