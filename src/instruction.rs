use borsh_derive::{BorshDeserialize, BorshSerialize};

use crate::store::{Amount, Price};

#[derive(Debug, BorshDeserialize, BorshSerialize)]
pub enum SplStoreInstruction {
    /// Initialize store and create ATA
    /// - \[writeable, signer] Funding account
    /// - \[writeable] Store ATA
    /// - [] Store account - wallet address
    /// - [] Token Mint account
    /// - [] System program account
    /// - [] SPL Token program account
    Initialize(Price),
    /// Buy tokens from a client
    /// - \[writeable, signer] Funding account - for ATA
    /// - \[writeable] Store account (sol source) - wallet
    /// - \[writeable] Store ATA (token recipient)
    /// - \[writeable, signer] Client account (sol recipient) - ATA's order/delegate
    /// - \[writeable] Client ATA (token source)
    /// - [] Token Mint account
    /// - [] System program account
    /// - [] SPL Token Program account
    Buy(Amount),
    /// Sell tokens to a client
    /// - \[writeable, signer] Funding account - for ATA
    /// - \[writeable, signer] Store account (sol recipient) - ATA's order/delegate
    /// - \[writeable] Store ATA (token source)
    /// - \[writeable] Client account (sol source) - wallet
    /// - \[writeable] Client ATA (token recipient)
    /// - [] Token Mint account
    /// - [] System program account
    /// - [] Token Program account
    Sell(Amount),
    /// Update token price
    /// - \[writeable] Store account
    UpdatePrice(Price),
}
