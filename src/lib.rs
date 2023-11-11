use entrypoint::process_instruction;

pub mod entrypoint;
pub mod error;
pub mod instruction;
mod macros;
pub mod processor;
pub mod store;
mod utils;

spl_token::solana_program::entrypoint!(process_instruction);
