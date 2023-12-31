use borsh::BorshDeserialize;
use eyre::eyre;
use solana_program_test::{processor, tokio, BanksClient, ProgramTest};
use solana_sdk::{
    hash::Hash,
    instruction::{AccountMeta, Instruction},
    message::Message,
    program_pack::Pack,
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    system_program,
    transaction::Transaction,
};
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account,
};
use spl_token::{
    instruction::initialize_mint,
    state::{Account, Mint},
    ui_amount_to_amount,
};

use spl_store::{
    entrypoint::process_instruction,
    instruction::SplStoreInstruction,
    store::{account::StoreAccount, Price},
};

async fn create_token_mint(
    banks_client: &mut BanksClient,
    recent_blockhash: Hash,
    payer: &Keypair,
    token_mint: &Keypair,
    token_program: &Pubkey,
    mint_auth: &Pubkey,
    lamports: u64,
    decimals: u8,
) -> eyre::Result<()> {
    let token_mint_account_ix = solana_sdk::system_instruction::create_account(
        &payer.pubkey(),
        &token_mint.pubkey(),
        lamports,
        Mint::LEN as u64,
        token_program,
    );

    let token_mint_init_ix = initialize_mint(
        token_program,
        &token_mint.pubkey(),
        mint_auth,
        None,
        decimals,
    )?;

    let token_mint_tx = Transaction::new_signed_with_payer(
        &[token_mint_account_ix, token_mint_init_ix],
        Some(&payer.pubkey()),
        &[payer, token_mint],
        recent_blockhash,
    );

    banks_client.process_transaction(token_mint_tx).await?;

    Ok(())
}

async fn mint_amount(
    banks_client: &mut BanksClient,
    recent_blockhash: Hash,
    token_program: &Pubkey,
    account: &Pubkey,
    mint: &Pubkey,
    mint_authority: &Keypair,
    payer: &Keypair,
    ui_amount: f64,
    mint_decimals: u8,
) -> eyre::Result<()> {
    let mint_lamports = ui_amount_to_amount(ui_amount, mint_decimals);
    let mint_ix = spl_token::instruction::mint_to(
        token_program,
        mint,
        account,
        &mint_authority.pubkey(),
        &[&payer.pubkey(), &mint_authority.pubkey()],
        mint_lamports,
    )?;

    let mint_tx = Transaction::new_signed_with_payer(
        &[mint_ix],
        Some(&payer.pubkey()),
        &[payer, mint_authority],
        recent_blockhash,
    );

    banks_client.process_transaction(mint_tx).await?;

    Ok(())
}

async fn unpack_account_data(
    banks_client: &mut BanksClient,
    pubkey: Pubkey,
) -> eyre::Result<Account> {
    let acc = banks_client
        .get_account(pubkey)
        .await?
        .ok_or(eyre!("get_account() failed"))?;
    let acc_data = spl_token::state::Account::unpack(&acc.data)?;
    Ok(acc_data)
}

async fn fetch_account_info_data<T: BorshDeserialize>(
    banks_client: &mut BanksClient,
    pubkey: Pubkey,
) -> eyre::Result<T> {
    let account = banks_client
        .get_account(pubkey)
        .await?
        .ok_or(eyre!("get_account"))?;
    let account_data = T::try_from_slice(&account.data)?;
    Ok(account_data)
}

#[tokio::test]
async fn it_works() {
    dotenv::dotenv().ok();

    // Setup keys ===============================================================

    let program_id = Pubkey::new_unique();

    let store = Keypair::new();
    let client = Keypair::new();
    let token_mint = Keypair::new();
    let admin = Keypair::new();

    let store_ata_pubkey = get_associated_token_address(&store.pubkey(), &token_mint.pubkey());

    let client_ata_pubkey = get_associated_token_address(&client.pubkey(), &token_mint.pubkey());

    let system_program_pubkey = system_program::id();
    let spl_token_program_pubkey = spl_token::id();
    let ata_program_pubkey = spl_associated_token_account::id();

    let auth = Keypair::new();

    let token_mint_decimals = 9;

    // Setup program test ==============================================================

    let mut program_test = ProgramTest::new(
        "spl-store", // Run the BPF version with `cargo test-bpf`
        program_id,
        processor!(process_instruction), // Run the native version with `cargo test`
    );

    // Add accounts ====================================================================

    program_test.add_account(
        client.pubkey(),
        solana_sdk::account::Account {
            lamports: 69_000_000_000,
            owner: program_id,
            ..Default::default()
        },
    );

    program_test.add_account(
        auth.pubkey(),
        solana_sdk::account::Account {
            lamports: 1_000_000_000,
            owner: system_program_pubkey,
            ..Default::default()
        },
    );

    // Setup test, calculate rent, create token mint ===============================

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // let rent = banks_client.get_rent().await.unwrap();
    // let mint_rent = rent.minimum_balance(Mint::LEN);

    create_token_mint(
        &mut banks_client,
        recent_blockhash,
        &payer,
        &token_mint,
        &spl_token_program_pubkey,
        &auth.pubkey(),
        ui_amount_to_amount(9_000f64, token_mint_decimals),
        token_mint_decimals,
    )
    .await
    .unwrap();

    // Create client ATA ==========================================================

    let client_ata_create_ix = create_associated_token_account(
        &payer.pubkey(),
        &client.pubkey(),
        &token_mint.pubkey(),
        &spl_token_program_pubkey,
    );

    let transaction = Transaction::new_signed_with_payer(
        &[client_ata_create_ix],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );

    banks_client.process_transaction(transaction).await.unwrap();

    assert_eq!(
        banks_client.get_balance(client_ata_pubkey).await.unwrap(),
        2_039_280
    );

    // Mint tokens to client ATA ===========================================

    mint_amount(
        &mut banks_client,
        recent_blockhash,
        &spl_token_program_pubkey,
        &client_ata_pubkey,
        &token_mint.pubkey(),
        &auth,
        &payer,
        14.,
        token_mint_decimals,
    )
    .await
    .unwrap();

    // Initialize store and check initial price ===================================

    let initial_price = std::env::var("TOKEN_INITIAL_PRICE")
        .unwrap()
        .parse::<Price>()
        .unwrap();

    let mut transaction = Transaction::new_with_payer(
        &[Instruction::new_with_borsh(
            program_id,
            &SplStoreInstruction::Initialize(initial_price, 32_000_200_000_000),
            vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(store_ata_pubkey, false),
                AccountMeta::new(store.pubkey(), true),
                AccountMeta::new(token_mint.pubkey(), false),
                AccountMeta::new(system_program_pubkey, false),
                AccountMeta::new(spl_token_program_pubkey, false),
                AccountMeta::new(admin.pubkey(), false),
                AccountMeta::new(ata_program_pubkey, false),
            ],
        )],
        Some(&payer.pubkey()),
    );

    transaction.sign(
        &[&payer, &store],
        banks_client.get_latest_blockhash().await.unwrap(),
    );
    banks_client.process_transaction(transaction).await.unwrap();

    let acc = fetch_account_info_data::<StoreAccount>(&mut banks_client, store.pubkey())
        .await
        .unwrap();

    assert_eq!(acc.price, initial_price);

    // Mint tokens to store ATA ===============================================

    mint_amount(
        &mut banks_client,
        recent_blockhash,
        &spl_token_program_pubkey,
        &store_ata_pubkey,
        &token_mint.pubkey(),
        &auth,
        &payer,
        14.,
        token_mint_decimals,
    )
    .await
    .unwrap();

    // Update token price ===============================================================

    let instruction = Instruction::new_with_borsh(
        program_id,
        &SplStoreInstruction::UpdatePrice(37),
        vec![
            AccountMeta::new(store.pubkey(), false),
            AccountMeta::new(admin.pubkey(), false),
        ],
    );

    let message = Message::new(&[instruction], Some(&payer.pubkey()));

    let mut transaction = Transaction::new(&[&payer], message, recent_blockhash);

    transaction.sign(
        &[&payer],
        banks_client.get_latest_blockhash().await.unwrap(),
    );
    banks_client.process_transaction(transaction).await.unwrap();

    let acc = fetch_account_info_data::<StoreAccount>(&mut banks_client, store.pubkey())
        .await
        .unwrap();

    assert_eq!(acc.price, 37);

    // Buy some tokens =============================================================

    let amount = 14;

    let transaction = Transaction::new_signed_with_payer(
        &[Instruction::new_with_borsh(
            program_id,
            &SplStoreInstruction::Buy(amount),
            vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(store.pubkey(), false),
                AccountMeta::new(store_ata_pubkey, false),
                AccountMeta::new(client.pubkey(), true),
                AccountMeta::new(client_ata_pubkey, false),
                AccountMeta::new(token_mint.pubkey(), false),
                AccountMeta::new(system_program_pubkey, false),
                AccountMeta::new(spl_token_program_pubkey, false),
                AccountMeta::new(ata_program_pubkey, false),
            ],
        )],
        Some(&payer.pubkey()),
        &[&payer, &client],
        banks_client.get_latest_blockhash().await.unwrap(),
    );

    banks_client.process_transaction(transaction).await.unwrap();

    assert_eq!(
        banks_client.get_balance(store.pubkey()).await.unwrap(),
        31_482_201_169_280
    );
    let client_acc_data = unpack_account_data(&mut banks_client, client_ata_pubkey)
        .await
        .unwrap();
    assert_eq!(client_acc_data.amount, 13_999_999_986);
    let store_acc_data = unpack_account_data(&mut banks_client, store_ata_pubkey)
        .await
        .unwrap();
    assert_eq!(store_acc_data.amount, 14_000_000_014);

    // Sell some tokens =============================================================

    let amount = 7;

    let transaction = Transaction::new_signed_with_payer(
        &[Instruction::new_with_borsh(
            program_id,
            &SplStoreInstruction::Sell(amount),
            vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(store.pubkey(), true),
                AccountMeta::new(store_ata_pubkey, false),
                AccountMeta::new(client.pubkey(), false),
                AccountMeta::new(client_ata_pubkey, false),
                AccountMeta::new(token_mint.pubkey(), false),
                AccountMeta::new(system_program_pubkey, false),
                AccountMeta::new(spl_token_program_pubkey, false),
                AccountMeta::new(ata_program_pubkey, false),
            ],
        )],
        Some(&payer.pubkey()),
        &[&payer, &store],
        banks_client.get_latest_blockhash().await.unwrap(),
    );

    banks_client.process_transaction(transaction).await.unwrap();

    assert_eq!(
        banks_client.get_balance(store.pubkey()).await.unwrap(),
        31_741_201_169_280
    );
    let client_acc_data = unpack_account_data(&mut banks_client, client_ata_pubkey)
        .await
        .unwrap();
    assert_eq!(client_acc_data.amount, 13_999_999_993);
    let store_acc_data = unpack_account_data(&mut banks_client, store_ata_pubkey)
        .await
        .unwrap();
    assert_eq!(store_acc_data.amount, 14_000_000_007);
}
