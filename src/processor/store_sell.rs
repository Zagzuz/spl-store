use spl_associated_token_account::{
    get_associated_token_address,
    instruction::create_associated_token_account,
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        msg,
        native_token::LAMPORTS_PER_SOL,
        program::invoke,
        program_error::ProgramError,
        pubkey::Pubkey,
    },
};
use spl_token::instruction::transfer;

use crate::{
    ensure,
    error::SplStoreError,
    store::{account::StoreAccount, Amount},
    utils::amount_to_lamports,
};

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], amount: Amount) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let store_account_info = next_account_info(accounts_iter)?;
    ensure!(
        store_account_info.owner == program_id,
        ProgramError::IncorrectProgramId
    );

    let store_ata_info = next_account_info(accounts_iter)?;
    let client_account_info = next_account_info(accounts_iter)?;
    let client_ata_info = next_account_info(accounts_iter)?;
    let token_mint_account_info = next_account_info(accounts_iter)?;
    let token_program_account_info = next_account_info(accounts_iter)?;

    ensure!(
        store_account_info.is_signer,
        SplStoreError::AccountNotSigner.into()
    );
    ensure!(
        store_ata_info.is_writable,
        SplStoreError::AccountNotWritable.into()
    );
    ensure!(
        client_ata_info.is_writable,
        SplStoreError::AccountNotWritable.into()
    );

    let token_lamports = amount_to_lamports(token_mint_account_info, amount)?;
    ensure!(
        store_ata_info.lamports() >= token_lamports,
        SplStoreError::InsufficientFundsForTransaction.into()
    );

    if client_ata_info.lamports() == 0 {
        msg!("Creating client (recipient) ATA...");
        ensure!(
            client_account_info.is_signer,
            SplStoreError::AccountNotSigner.into()
        );
        let system_program_account = next_account_info(accounts_iter)?;
        let spl_token_program_account = next_account_info(accounts_iter)?;
        let create_ata_ix = create_associated_token_account(
            &client_account_info.key,
            &client_account_info.key,
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
                client_account_info.clone(),
                client_ata_info.clone(),
                client_account_info.clone(),
                token_mint_account_info.clone(),
                system_program_account.clone(),
                spl_token_program_account.clone(),
            ],
        )?;
    }

    let price = StoreAccount::get_price(store_account_info)?;
    msg!("Price: {} SOL", price);
    let sol_amount = amount * price;
    let sol_lamports = (sol_amount * LAMPORTS_PER_SOL as f64) as u64;
    ensure!(
        client_account_info.lamports() >= sol_lamports,
        SplStoreError::InsufficientFundsForTransaction.into()
    );

    ensure!(
        get_associated_token_address(&store_account_info.key, &token_mint_account_info.key)
            == *store_ata_info.key,
        SplStoreError::InvalidAtaAddress.into()
    );
    ensure!(
        get_associated_token_address(&client_account_info.key, &token_mint_account_info.key)
            == *client_ata_info.key,
        SplStoreError::InvalidAtaAddress.into()
    );

    let transfer_ix = transfer(
        &token_program_account_info.key,
        &store_ata_info.key,
        &client_ata_info.key,
        &store_account_info.key,
        &[&store_account_info.key],
        token_lamports,
    )?;
    // [writable] The source account.
    // [writable] The destination account.
    // [signer] The source account’s owner/delegate.
    invoke(
        &transfer_ix,
        &[
            store_ata_info.clone(),
            client_ata_info.clone(),
            store_account_info.clone(),
        ],
    )?;
    msg!("Store ATA ==[{} tokens]==> Client ATA", amount);

    **client_account_info.try_borrow_mut_lamports()? -= sol_lamports;
    **store_account_info.try_borrow_mut_lamports()? += sol_lamports;
    msg!("Client Account ==[{} SOL]==> Store Account", sol_amount);
    Ok(())
}
