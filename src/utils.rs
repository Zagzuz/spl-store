use spl_associated_token_account::solana_program::{
    account_info::AccountInfo, program_error::ProgramError, program_pack::Pack,
};
use spl_token::{
    solana_program::entrypoint::ProgramResult,
    state::{Account, GenericTokenAccount},
    ui_amount_to_amount,
};

use crate::{error::SplStoreError, store::Amount};

pub fn amount_to_lamports(mint: &AccountInfo, amount: Amount) -> Result<u64, ProgramError> {
    let mint_account_data = spl_token::state::Mint::unpack_from_slice(&mint.try_borrow_data()?)?;
    let mint_decimals = mint_account_data.decimals;
    Ok(ui_amount_to_amount(amount, mint_decimals))
}

pub fn check_ata_mint(ata_info: &AccountInfo, token_mint: &AccountInfo) -> ProgramResult {
    match Account::unpack_account_mint(&ata_info.data.borrow()) {
        None => Err(SplStoreError::NoAccountMint.into()),
        Some(client_ata_mint) if client_ata_mint != token_mint.key => {
            Err(SplStoreError::WrongAccountMint.into())
        }
        _ => Ok(()),
    }
}
