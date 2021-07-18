use solana_program::pubkey::Pubkey;

/// marker for keys which are programs
pub type ProgramPubkey = Pubkey;

/// marker for addresses which are derived from program (so these such accounts can only be created and initialized by the owner program)
pub type ProgramDerivedPubkey = Pubkey;


/// related to [solana_program::clock::UnixTimestamp]
pub type ApproximateSeconds = i64;

/// Fixed point with one = 1.0000.
pub type BasisPointsMultiplier = u32;

pub type MintPubkey = Pubkey;
pub type TokenAccountPubKey = Pubkey;
pub type SignerPubkey = Pubkey;
pub type TokenAmount = u64;