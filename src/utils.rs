use spl_associated_token_account::solana_program::account_info::AccountInfo;
use spl_token::{
    solana_program::entrypoint::ProgramResult,
    state::{Account, GenericTokenAccount},
};

use crate::error::SplStoreError;

pub fn check_ata_mint(ata_info: &AccountInfo, token_mint: &AccountInfo) -> ProgramResult {
    match Account::unpack_account_mint(&ata_info.data.borrow()) {
        None => Err(SplStoreError::NoAccountMint.into()),
        Some(client_ata_mint) if client_ata_mint != token_mint.key => {
            Err(SplStoreError::WrongAccountMint.into())
        }
        _ => Ok(()),
    }
}
