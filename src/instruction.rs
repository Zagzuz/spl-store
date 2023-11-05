use crate::store::{Amount, Price};
use borsh_derive::{BorshDeserialize, BorshSerialize};

#[derive(Debug, BorshDeserialize, BorshSerialize)]
pub enum SplStoreInstruction {
    /// Initialize store and create ATA
    /// - \[writeable, signer] Store account - ATA funding and wallet
    /// - \[writeable] Store ATA
    /// - [] Token Mint account
    /// - [] Token program account
    /// - [] System program account
    /// - [] SPL Token program account
    Initialize(Price),
    /// Buy tokens from a client
    /// - \[writeable, signer] Store account (sol source) - ATA funding and wallet
    /// - \[writeable] Store ATA (token recipient)
    /// - \[writeable, signer] Client account (sol recipient) - ATA's order/delegate
    /// - \[writeable] Client ATA (token source)
    /// - [] Token Mint account
    /// - [] Token Program account
    Buy(Amount),
    /// Sell tokens to a client
    /// - \[writeable, signer] Store account (sol recipient) - ATA's order/delegate
    /// - \[writeable] Store ATA (token source)
    /// - \[writeable, signer] Client account (sol source) - ATA funding and wallet
    /// - \[writeable] Client ATA (token recipient)
    /// - [] Token Mint account
    /// - [] Token Program account
    Sell(Amount),
    /// Update token price
    /// - \[writeable] Store account
    UpdatePrice(Price),
}
