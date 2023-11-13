use borsh::BorshDeserialize;
use spl_token::solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey,
};

use crate::instruction::SplStoreInstruction;

mod store_buy;
mod store_initialize;
mod store_sell;
mod store_update_price;

pub struct Processor;

impl Processor {
    pub fn process_instruction(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = SplStoreInstruction::try_from_slice(instruction_data)?;
        match instruction {
            SplStoreInstruction::Initialize(price, add_sol) => {
                store_initialize::process(program_id, accounts, price, add_sol)
            }
            SplStoreInstruction::Buy(token_amount) => {
                store_buy::process(program_id, accounts, token_amount)
            }
            SplStoreInstruction::UpdatePrice(new_price) => {
                store_update_price::process(program_id, accounts, new_price)
            }
            SplStoreInstruction::Sell(token_amount) => {
                store_sell::process(program_id, accounts, token_amount)
            }
        }
    }
}
