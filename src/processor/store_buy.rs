use spl_associated_token_account::{
    get_associated_token_address,
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        msg,
        native_token::LAMPORTS_PER_SOL,
        program::invoke,
        pubkey::Pubkey,
    },
};
use spl_token::{instruction::transfer, solana_program::program_error::ProgramError};

use crate::{
    ensure,
    error::SplStoreError,
    store::{account::StoreAccount, Amount},
    utils::{amount_to_lamports, check_ata_mint},
};

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], amount: Amount) -> ProgramResult {
    let accounts_info_iter = &mut accounts.iter();

    let funding_account_info = next_account_info(accounts_info_iter)?;
    let store_account_info = next_account_info(accounts_info_iter)?;
    let store_ata_info = next_account_info(accounts_info_iter)?;
    let client_account_info = next_account_info(accounts_info_iter)?;
    let client_ata_info = next_account_info(accounts_info_iter)?;
    let token_mint_account_info = next_account_info(accounts_info_iter)?;
    let system_program_account_info = next_account_info(accounts_info_iter)?;
    let spl_token_program_account_info = next_account_info(accounts_info_iter)?;

    ensure!(
        spl_token::check_id(spl_token_program_account_info.key),
        ProgramError::IncorrectProgramId
    );
    ensure!(
        store_account_info.owner == program_id,
        ProgramError::IncorrectProgramId
    );
    ensure!(
        client_account_info.is_signer,
        SplStoreError::AccountNotSigner.into()
    );
    ensure!(
        client_ata_info.is_writable,
        SplStoreError::AccountNotWritable.into()
    );
    ensure!(
        store_ata_info.is_writable,
        SplStoreError::AccountNotWritable.into()
    );

    let token_lamports = amount_to_lamports(token_mint_account_info, amount)?;
    msg!(
        "token lamports: {}, client ata lamports: {}",
        token_lamports,
        client_ata_info.lamports()
    );
    ensure!(
        client_ata_info.lamports() >= token_lamports,
        SplStoreError::InsufficientFundsForTransaction.into()
    );

    check_ata_mint(client_ata_info, token_mint_account_info)?;

    if store_ata_info.lamports() == 0 {
        msg!("Creating store (recipient) ATA...");
        StoreAccount::initialize_ata(&[
            funding_account_info.clone(),
            store_ata_info.clone(),
            store_account_info.clone(),
            token_mint_account_info.clone(),
            system_program_account_info.clone(),
            spl_token_program_account_info.clone(),
        ])?;
    }

    check_ata_mint(store_ata_info, token_mint_account_info)?;

    let price = StoreAccount::get_price(store_account_info)?;
    msg!("Price: {} SOL", price);
    let sol_amount = amount * price;
    let sol_lamports = (sol_amount * LAMPORTS_PER_SOL as f64) as u64;
    ensure!(
        store_account_info.lamports() >= sol_lamports,
        SplStoreError::InsufficientFundsForTransaction.into()
    );

    ensure!(
        get_associated_token_address(store_account_info.key, token_mint_account_info.key)
            == *store_ata_info.key,
        SplStoreError::InvalidAtaAddress.into()
    );
    ensure!(
        get_associated_token_address(client_account_info.key, token_mint_account_info.key)
            == *client_ata_info.key,
        SplStoreError::InvalidAtaAddress.into()
    );

    let transfer_ix = transfer(
        spl_token_program_account_info.key,
        client_ata_info.key,
        store_ata_info.key,
        client_account_info.key,
        &[&client_account_info.key],
        0,
    )?;
    // [writable] The source account.
    // [writable] The destination account.
    // [signer] The source account’s owner/delegate.
    invoke(
        &transfer_ix,
        &[
            client_ata_info.clone(),
            store_ata_info.clone(),
            client_account_info.clone(),
        ],
    )?;
    msg!("Client ATA ==[{} tokens]==> Store ATA", amount);

    **store_account_info.try_borrow_mut_lamports()? -= sol_lamports;
    **client_account_info.try_borrow_mut_lamports()? += sol_lamports;
    msg!("Store Account ==[{} SOL]==> Client Account", sol_amount);

    Ok(())
}
