use borsh::{BorshDeserialize, BorshSerialize};
use borsh_derive::{BorshDeserialize, BorshSerialize};
use spl_associated_token_account::{
    instruction::create_associated_token_account,
    solana_program::{account_info::next_account_info, program::invoke},
};
use spl_token::solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
};

use crate::{ensure, error::SplStoreError, store::Price};

#[derive(Debug, Default, BorshSerialize, BorshDeserialize)]
pub struct StoreAccount {
    pub price: Price,
}

impl StoreAccount {
    pub fn update_price(account_info: &AccountInfo, new_price: Price) -> ProgramResult {
        ensure!(new_price > 0 as Price, SplStoreError::InvalidPrice.into());
        let mut store_account = StoreAccount::try_from_slice(&account_info.data.borrow())?;
        store_account.price = new_price;
        store_account.serialize(&mut &mut account_info.data.borrow_mut()[..])?;
        Ok(())
    }

    pub fn get_price(account_info: &AccountInfo) -> Result<Price, ProgramError> {
        let store_account = StoreAccount::try_from_slice(&account_info.data.borrow())?;
        Ok(store_account.price)
    }

    /// - [writeable, signer] Store account
    /// - [writeable] Store ATA
    /// - [] Token Mint account
    /// - [] Token program account
    /// - [] System program account
    /// - [] SPL Token program account
    pub fn initialize_ata(accounts: &[AccountInfo]) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();

        let store_account_info = next_account_info(accounts_iter)?;
        let store_ata_info = next_account_info(accounts_iter)?;
        let token_mint_account_info = next_account_info(accounts_iter)?;
        let token_program_account_info = next_account_info(accounts_iter)?;
        let system_program_account = next_account_info(accounts_iter)?;
        let spl_token_program_account = next_account_info(accounts_iter)?;

        ensure!(
            store_account_info.is_writable,
            SplStoreError::AccountNotWritable.into()
        );
        ensure!(
            store_account_info.is_signer,
            SplStoreError::AccountNotSigner.into()
        );
        ensure!(
            store_ata_info.is_writable,
            SplStoreError::AccountNotWritable.into()
        );

        let create_ata_ix = create_associated_token_account(
            &store_account_info.key,
            &store_account_info.key,
            &token_mint_account_info.key,
            &token_program_account_info.key,
        );
        // [writeable,signer] Funding account (must be a system account)
        // [writeable] Associated token account address to be created
        // [] Wallet address for the new associated token account
        // [] The token mint for the new associated token account
        // [] System program
        // [] SPL Token program
        invoke(
            &create_ata_ix,
            &[
                store_account_info.clone(),
                store_ata_info.clone(),
                store_account_info.clone(),
                token_mint_account_info.clone(),
                system_program_account.clone(),
                spl_token_program_account.clone(),
            ],
        )
    }
}
