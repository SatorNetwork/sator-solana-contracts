use solana_program::pubkey::Pubkey;

/// marker for keys which are programs
pub type ProgramPubkey = Pubkey;

/// marker for addresses which are derived from program (so these such accounts can only be created and initialized by the owner program)
pub type ProgramDerivedPubkey = Pubkey;

/// related to [solana_program::clock::UnixTimestamp]
pub type ApproximateSeconds = i64;

pub type MintPubkey = Pubkey;
pub type TokenAccountPubkey = Pubkey;
pub type SignerPubkey = Pubkey;
pub type TokenAmount = u64;

pub type Lamports = u64;
