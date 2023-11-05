use crate::{
    ensure,
    store::{account::StoreAccount, Price},
};
use spl_associated_token_account::solana_program::msg;
use spl_token::solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint_deprecated::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], price: Price) -> ProgramResult {
    msg!("Store initialization");
    let accounts_iter = &mut accounts.iter();

    let store_account_info = next_account_info(accounts_iter)?;
    let store_ata_info = next_account_info(accounts_iter)?;

    ensure!(
        store_account_info.owner == program_id,
        ProgramError::IncorrectProgramId
    );
    ensure!(
        store_account_info.lamports() != 0,
        ProgramError::UninitializedAccount
    );
    StoreAccount::update_price(&store_account_info, price)?;
    msg!("Token initial price set to {}", price);

    if store_ata_info.lamports() == 0 {
        StoreAccount::initialize_ata(accounts)?;
    }
    msg!("Store ATA initialized");
    Ok(())
}
