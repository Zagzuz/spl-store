use borsh_derive::{BorshDeserialize, BorshSerialize};
use spl_associated_token_account::{
    get_associated_token_address,
    instruction::create_associated_token_account,
    solana_program::{account_info::next_account_info, program::invoke},
};
use spl_token::solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    system_program,
};

use crate::{ensure, error::SplStoreError, store::Price};

#[derive(Debug, Default, BorshSerialize, BorshDeserialize)]
pub struct StoreAccount {
    pub price: Price,
}

impl StoreAccount {
    pub fn update_price(account_info: &AccountInfo, new_price: Price) -> ProgramResult {
        ensure!(new_price > 0 as Price, SplStoreError::InvalidPrice.into());
        let mut store_account: StoreAccount =
            borsh::BorshDeserialize::try_from_slice(&account_info.data.borrow())?;
        store_account.price = new_price;
        borsh::BorshSerialize::serialize(
            &store_account,
            &mut &mut account_info.data.borrow_mut()[..],
        )?;
        Ok(())
    }

    pub fn get_price(account_info: &AccountInfo) -> Result<Price, ProgramError> {
        let store_account: StoreAccount =
            borsh::BorshDeserialize::try_from_slice(&account_info.data.borrow())?;
        Ok(store_account.price)
    }

    /// - \[writeable, signer] Funding account
    /// - \[writeable] Store ATA
    /// - [] Store account - wallet address
    /// - [] Token Mint account
    /// - [] Token program account
    /// - [] System program account
    /// - [] SPL Token program account
    pub fn initialize_ata(account_infos: &[AccountInfo]) -> ProgramResult {
        let account_infos_iter = &mut account_infos.iter();

        let funding_account_info = next_account_info(account_infos_iter)?;
        let ata_account_info = next_account_info(account_infos_iter)?;
        let wallet_account_info = next_account_info(account_infos_iter)?;
        let token_mint_account_info = next_account_info(account_infos_iter)?;
        let system_program_account_info = next_account_info(account_infos_iter)?;
        let spl_token_program_account_info = next_account_info(account_infos_iter)?;

        let expected_ata_pubkey =
            get_associated_token_address(&wallet_account_info.key, &token_mint_account_info.key);

        ensure!(
            system_program::check_id(&system_program_account_info.key),
            ProgramError::IncorrectProgramId.into()
        );

        ensure!(
            &expected_ata_pubkey == ata_account_info.key,
            SplStoreError::UnexpectedAtaAddress.into()
        );
        ensure!(
            funding_account_info.is_writable,
            SplStoreError::AccountNotWritable.into()
        );
        ensure!(
            funding_account_info.is_signer,
            SplStoreError::AccountNotSigner.into()
        );
        ensure!(
            ata_account_info.is_writable,
            SplStoreError::AccountNotWritable.into()
        );

        let create_ata_ix = create_associated_token_account(
            &funding_account_info.key,
            &wallet_account_info.key,
            &token_mint_account_info.key,
            &spl_token_program_account_info.key,
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
                funding_account_info.clone(),
                ata_account_info.clone(),
                wallet_account_info.clone(),
                token_mint_account_info.clone(),
                system_program_account_info.clone(),
                spl_token_program_account_info.clone(),
            ],
        )
    }
}
