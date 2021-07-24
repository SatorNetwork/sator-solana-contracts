use num_derive::ToPrimitive;
use num_traits::ToPrimitive;
use solana_program::{entrypoint::ProgramResult, program_error::ProgramError};

#[derive(Debug, ToPrimitive)]
pub enum Error {
    LockStakingTimeMustBeMoreThanMinimal,
    UnlockCanBeDoneOnlyAfterStakeTimeLapsed,
    UnlockLockAccountNotDerivedFromWalletStakeProgram,
}

impl Error {
    pub fn to_error_code(&self) -> u32 {
        self.to_u32().unwrap()
    }
}

impl Into<ProgramResult> for Error {
    fn into(self) -> ProgramResult {
        Err(ProgramError::Custom(self.to_error_code()))
    }
}
