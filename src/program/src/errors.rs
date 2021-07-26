use num_derive::ToPrimitive;
use num_traits::ToPrimitive;
use solana_program::{entrypoint::ProgramResult, program_error::ProgramError};

#[derive(Debug, ToPrimitive)]
pub enum Error {
    StakeStakingTimeMustBeMoreThanMinimal,
    StakeStakingTimeMustBeMoreThanPrevious,
    UnstakeCanBeDoneOnlyAfterStakeTimeLapsed,
    UnstakeStakeAccountNotDerivedFromWalletStakeProgram,
    UnstakeOverflow,
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

impl Into<ProgramError> for Error {
    fn into(self) -> ProgramError {
        ProgramError::Custom(self.to_error_code())
    }
}
