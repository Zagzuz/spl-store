use spl_token::solana_program::{decode_error::DecodeError, program_error::ProgramError};
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq, Copy)]
pub enum SplStoreError {
    #[error("Account is not writable")]
    AccountNotWritable,
    #[error("Account is not signer")]
    AccountNotSigner,
    #[error("Account is signer, but should not be")]
    AccountSigner,
    #[error("Insufficient funds for transaction")]
    InsufficientFundsForTransaction,
    #[error("No initial price")]
    NoInitialPrice,
    #[error("Invalid price")]
    InvalidPrice,
    #[error("Invalid associated token account address")]
    InvalidAtaAddress,
    #[error("No initial lamports")]
    NoInitialLamports,
    #[error("Wrong initial lamports")]
    WrongInitialLamports,
    #[error("Failed to unpack account mint for ATA")]
    NoAccountMint,
    #[error("Wrong account mint")]
    WrongAccountMint,
    #[error("Unexpected ATA address")]
    UnexpectedAtaAddress,
}

impl From<SplStoreError> for ProgramError {
    fn from(value: SplStoreError) -> Self {
        ProgramError::Custom(value as u32)
    }
}

impl<T> DecodeError<T> for SplStoreError {
    fn type_of() -> &'static str {
        "SPL Store error"
    }
}
