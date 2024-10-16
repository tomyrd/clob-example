use core::panic;
use std::time::Duration;

use clap::Parser;
use miden_client::{
    accounts::{Account, AccountId, AccountStorageType, AccountTemplate},
    assets::{FungibleAsset, TokenSymbol},
    auth::TransactionAuthenticator,
    crypto::FeltRng,
    notes::NoteType,
    rpc::NodeRpcClient,
    store::Store,
    transactions::{build_swap_tag, request::TransactionRequest},
    Client, Word,
};
use tokio::time::sleep;

use crate::{
    constants::DB_FILE_PATH,
    utils::{clear_notes_tables, create_swap_notes_transaction_request},
};

// Setup COMMAND
// ================================================================================================

#[derive(Debug, Clone, Parser)]
#[clap(about = "Setup the order book")]
pub struct SetupCmd {}

impl SetupCmd {
    pub async fn execute<N: NodeRpcClient, R: FeltRng, S: Store, A: TransactionAuthenticator>(
        &self,
        mut client: Client<N, R, S, A>,
    ) -> Result<(), String> {
        // Sync rollup state
        client.sync_state().await.unwrap();

        // Create faucet accounts
        let (faucet1, _) = Self::create_faucet(1000, "ASSETA", &mut client);
        let (faucet2, _) = Self::create_faucet(1000, "ASSETB", &mut client);

        // Create user account
        let (user, _) = Self::create_wallet(&mut client);

        // Mint assets for user
        Self::fund_user_wallet(
            faucet1.id(),
            1000,
            faucet2.id(),
            1000,
            user.id(),
            &mut client,
        )
        .await;

        // Create 50 ASSETA/ASSETB swap notes
        Self::create_swap_notes(
            50,
            faucet1.id(),
            500,
            faucet2.id(),
            500,
            user.id(),
            &mut client,
        )
        .await;

        // Create 50 ASSETB/ASSETA swap notes
        Self::create_swap_notes(
            50,
            faucet2.id(),
            500,
            faucet1.id(),
            500,
            user.id(),
            &mut client,
        )
        .await;

        clear_notes_tables(DB_FILE_PATH).unwrap();

        Self::print_clob_data(faucet1.id(), faucet2.id(), user.id());

        println!("CLOB successfully setup.");

        Ok(())
    }

    async fn create_swap_notes<
        N: NodeRpcClient,
        R: FeltRng,
        S: Store,
        A: TransactionAuthenticator,
    >(
        num_notes: u8,
        faucet1: AccountId,
        total_asset_offering: u64,
        faucet2: AccountId,
        total_asset_requesting: u64,
        user: AccountId,
        client: &mut Client<N, R, S, A>,
    ) {
        let transaction_request = create_swap_notes_transaction_request(
            num_notes,
            user,
            faucet1,
            total_asset_offering,
            faucet2,
            total_asset_requesting,
            client.rng(),
        )
        .unwrap();
        let tx_result = client.new_transaction(user, transaction_request).unwrap();
        client.submit_transaction(tx_result).await.unwrap();
    }

    async fn fund_user_wallet<
        N: NodeRpcClient,
        R: FeltRng,
        S: Store,
        A: TransactionAuthenticator,
    >(
        faucet1: AccountId,
        asset_a_amount: u64,
        faucet2: AccountId,
        asset_b_amount: u64,
        user: AccountId,
        client: &mut Client<N, R, S, A>,
    ) {
        // Setup mint
        let note_type = NoteType::Public;

        // Mint AssetA
        let asset_a = FungibleAsset::new(faucet1, asset_a_amount).unwrap();
        let transaction_request =
            TransactionRequest::mint_fungible_asset(asset_a, user, note_type, client.rng())
                .unwrap();
        let tx_result = client
            .new_transaction(faucet1, transaction_request)
            .unwrap();
        let asset_a_note_id = tx_result.relevant_notes()[0].id();
        client.submit_transaction(tx_result).await.unwrap();

        // Mint AssetB
        let asset_b = FungibleAsset::new(faucet2, asset_b_amount).unwrap();
        let transaction_request =
            TransactionRequest::mint_fungible_asset(asset_b, user, note_type, client.rng())
                .unwrap();
        let tx_result = client
            .new_transaction(faucet2, transaction_request)
            .unwrap();
        let asset_b_note_id = tx_result.relevant_notes()[0].id();
        client.submit_transaction(tx_result).await.unwrap();

        // Sync rollup state
        sleep(Duration::from_secs(20)).await;
        client.sync_state().await.unwrap();

        // Fund receiving wallet
        let tx_request = TransactionRequest::consume_notes(vec![asset_a_note_id, asset_b_note_id]);
        let tx_result = client.new_transaction(user, tx_request).unwrap();
        client.submit_transaction(tx_result).await.unwrap();
    }

    fn create_wallet<N: NodeRpcClient, R: FeltRng, S: Store, A: TransactionAuthenticator>(
        client: &mut Client<N, R, S, A>,
    ) -> (Account, Word) {
        let wallet_template = AccountTemplate::BasicWallet {
            mutable_code: false,
            storage_type: AccountStorageType::OnChain,
        };
        client.new_account(wallet_template).unwrap()
    }

    fn create_faucet<N: NodeRpcClient, R: FeltRng, S: Store, A: TransactionAuthenticator>(
        max_supply: u64,
        token_symbol: &str,
        client: &mut Client<N, R, S, A>,
    ) -> (Account, Word) {
        let faucet_template = AccountTemplate::FungibleFaucet {
            token_symbol: TokenSymbol::new(token_symbol).unwrap(),
            decimals: 10,
            max_supply,
            storage_type: AccountStorageType::OnChain,
        };
        client.new_account(faucet_template).unwrap()
    }

    fn print_clob_data(faucet1: AccountId, faucet2: AccountId, user: AccountId) {
        // build swap tags
        let faucet1_faucet2_tag = build_swap_tag(NoteType::Public, faucet1, faucet2).unwrap();
        let faucet2_faucet1_tag = build_swap_tag(NoteType::Public, faucet2, faucet1).unwrap();

        if faucet1_faucet2_tag == faucet2_faucet1_tag {
            panic!("Both asset tags should not be similar.")
        }

        println!("faucet1: {}", faucet1);
        println!("faucet2: {}\n", faucet2);
        println!("faucet1/faucet2 tag: {}", faucet1_faucet2_tag);
        println!("faucet2/faucet1 tag: {}\n", faucet2_faucet1_tag);
        println!("User: {}\n", user);
    }
}
