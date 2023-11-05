use crate::processor::Processor;
use spl_token::solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey,
};

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("EntryPoint");
    dotenv::dotenv().expect("dotenv init failure");
    Processor::process_instruction(program_id, accounts, instruction_data)
}
