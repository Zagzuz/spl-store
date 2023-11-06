use spl_associated_token_account::solana_program::entrypoint::ProgramResult;
use spl_token::solana_program::{
    account_info::{next_account_info, AccountInfo},
    pubkey::Pubkey,
};

use crate::{
    ensure,
    error::SplStoreError,
    store::{account::StoreAccount, Price},
};

pub fn process(_program_id: &Pubkey, accounts: &[AccountInfo], new_price: Price) -> ProgramResult {
    let accounts_info_iter = &mut accounts.iter();
    let account_info = next_account_info(accounts_info_iter)?;
    ensure!(
        account_info.is_writable,
        SplStoreError::AccountNotWritable.into()
    );
    StoreAccount::update_price(account_info, new_price)
}
