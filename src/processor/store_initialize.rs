use spl_associated_token_account::solana_program::msg;
use spl_token::solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint_deprecated::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{
    ensure,
    store::{account::StoreAccount, Amount, Price},
};

pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    price: Price,
    add_sol: Amount,
) -> ProgramResult {
    msg!("Store initialization");
    let accounts_info_iter = &mut accounts.iter();

    let funding_account_info = next_account_info(accounts_info_iter)?;
    let store_ata_info = next_account_info(accounts_info_iter)?;
    let store_account_info = next_account_info(accounts_info_iter)?;
    let token_mint_account_info = next_account_info(accounts_info_iter)?;
    let system_program_account_info = next_account_info(accounts_info_iter)?;
    let spl_token_program_account_info = next_account_info(accounts_info_iter)?;
    let admin_account_info = next_account_info(accounts_info_iter)?;

    if store_account_info.lamports() == 0 {
        StoreAccount::initialize_account(
            program_id,
            &[funding_account_info.clone(), store_account_info.clone()],
            add_sol,
        )?;
    }

    ensure!(
        store_account_info.owner == program_id,
        ProgramError::IncorrectProgramId
    );

    let mut store_account: StoreAccount =
        borsh::BorshDeserialize::try_from_slice(&store_account_info.data.borrow())?;

    store_account.admin = admin_account_info.key.clone();
    store_account.price = price;

    borsh::BorshSerialize::serialize(
        &store_account,
        &mut &mut store_account_info.data.borrow_mut()[..],
    )?;
    msg!("Token initial price set to {}", price);

    if store_ata_info.lamports() == 0 {
        msg!("Initializing store ATA...");
        StoreAccount::initialize_ata(&[
            funding_account_info.clone(),
            store_ata_info.clone(),
            store_account_info.clone(),
            token_mint_account_info.clone(),
            system_program_account_info.clone(),
            spl_token_program_account_info.clone(),
        ])?;
    }
    msg!("Store ATA initialized");
    Ok(())
}
