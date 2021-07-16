#![feature(trivial_bounds)]

mod state;
mod entrypoint;

use solana_program::{account_info::AccountInfo, pubkey::Pubkey};
use state::*;
