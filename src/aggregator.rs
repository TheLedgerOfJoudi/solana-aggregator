use crate::{database::Database, error::AggregatorError};
use chrono::{DateTime, Utc};
use futures_util::StreamExt;
use solana_client::{
    nonblocking::pubsub_client::PubsubClient, rpc_client::RpcClient, rpc_request::RpcRequest,
};
use solana_sdk::pubkey::Pubkey;
use solana_transaction_status::{
    EncodedConfirmedBlock, EncodedTransaction, EncodedTransactionWithStatusMeta, UiMessage,
    UiRawMessage, UiTransactionStatusMeta,
};
use std::{
    str::FromStr,
    thread,
    time::{Duration, UNIX_EPOCH},
};
use tokio::runtime::Handle;
const MAX_ITERATIONS: i32 = 100;

#[derive(serde::Deserialize)]
struct Env {
    ws_url: url::Url,
    rpc_url: url::Url,
}

#[derive(Debug)]
struct Transaction {
    sender: Pubkey,
    receiver: Pubkey,
    amount: i64,
    timestamp: String,
    signatures: Vec<String>,
}

impl Transaction {

    /// Creates a new, empty `Transaction`.
    fn new() -> Transaction {
        Transaction {
            sender: Pubkey::default(),
            receiver: Pubkey::default(),
            amount: 0,
            timestamp: "".to_string(),
            signatures: vec![],
        }
    }

    /// Processes an encoded transaction and extracts relevant information.
    ///
    /// # Arguments
    ///
    /// * `encoded_transaction` - The encoded transaction with status metadata.
    ///
    /// # Errors
    ///
    /// Returns `AggregatorError::MetaDataFetchError` if the metadata is missing.
    fn handle_transaction(
        &mut self,
        encoded_transaction: &EncodedTransactionWithStatusMeta,
    ) -> Result<(), AggregatorError> {
        let meta_data = match encoded_transaction.meta.as_ref() {
            Some(res) => res,
            None => return Err(AggregatorError::MetaDataFetchError),
        };

        if let EncodedTransaction::Json(message) = &encoded_transaction.transaction {
            let signatures = &message.signatures;
            self.signatures = signatures.to_vec();
            if let UiMessage::Raw(msg) = &message.message {
                self.fetch_sender(meta_data, msg);
                self.fetch_receiver(meta_data, msg);
                self.fetch_amount(meta_data, msg);
            }
        }
        Ok(())
    }

    /// Fetches the sender's public key from the transaction message.
    ///
    /// # Arguments
    ///
    /// * `_meta_data` - The transaction status metadata (unused).
    /// * `message` - The raw transaction message.
    fn fetch_sender(&mut self, _meta_data: &UiTransactionStatusMeta, message: &UiRawMessage) {
        let account_keys = &message.account_keys;
        let key = Pubkey::from_str(&account_keys[0]);
        self.sender = key.unwrap();
    }

    /// Fetches the receiver's public key from the transaction message.
    ///
    /// # Arguments
    ///
    /// * `_meta_data` - The transaction status metadata (unused).
    /// * `message` - The raw transaction message.
    fn fetch_receiver(&mut self, _meta_data: &UiTransactionStatusMeta, message: &UiRawMessage) {
        let account_keys = &message.account_keys;
        let key = Pubkey::from_str(&account_keys[1]);
        self.receiver = key.unwrap();
    }

    /// Fetches the transaction amount from the transaction metadata.
    ///
    /// # Arguments
    ///
    /// * `meta_data` - The transaction status metadata.
    /// * `_message` - The raw transaction message (unused).
    fn fetch_amount(&mut self, meta_data: &UiTransactionStatusMeta, _message: &UiRawMessage) {
        let amount = meta_data.pre_balances[0] as i64 - meta_data.post_balances[0] as i64;
        self.amount = amount;
    }

    /// Inserts the transaction into the database.
    ///
    /// # Arguments
    ///
    /// * `database` - The database instance.
    fn insert_to_database(&self, database: &mut Database) {
        let _ = database.insert(
            self.sender,
            self.receiver,
            self.amount,
            &self.timestamp,
            &self.signatures[0],
        );
    }
}

/// Aggregates data from the Solana blockchain by subscribing to new slots and processing transactions.
///
/// # Errors
///
/// Returns an `AggregatorError` if there is an error fetching environment variables, connecting to the Pubsub client,
/// subscribing to slots, or other runtime errors.
pub async fn aggregate_data() -> Result<(), AggregatorError> {
    let _ = Database::new();
    let env = match envy::from_env::<Env>() {
        Ok(res) => res,
        Err(_) => return Err(AggregatorError::EnvFetchError),
    };

    let pubsub = match PubsubClient::new(env.ws_url.as_ref()).await {
        Ok(res) => res,
        Err(_) => return Err(AggregatorError::PubsubClientError),
    };

    let (mut accounts, unsubscriber) = match pubsub.slot_subscribe().await {
        Ok(res) => res,
        Err(_) => return Err(AggregatorError::SlotSubscribeError),
    };

    for _ in 0..MAX_ITERATIONS {
        if let Some(response) = accounts.next().await {
            println!("{:?}", response);
            let handle = Handle::current();
            handle.spawn(async move { get_block(response.root).await });
        }
    }
    unsubscriber().await;
    Ok(())
}

/// Retrieves and processes a block from the Solana blockchain.
///
/// # Arguments
///
/// * `slot` - The slot number to fetch the block for.
///
/// # Errors
///
/// Returns an `AggregatorError` if there is an error connecting to the database, fetching environment variables,
/// sending the RPC request, or processing the block.
pub async fn get_block(slot: u64) -> Result<(), AggregatorError> {
    let mut database = match Database::new_connection() {
        Ok(res) => res,
        Err(_) => return Err(AggregatorError::DatabaseError),
    };

    let env = match envy::from_env::<Env>() {
        Ok(res) => res,
        Err(_) => return Err(AggregatorError::EnvFetchError),
    };
    let rpc = RpcClient::new(env.rpc_url.to_string());
    let ten_millis = Duration::from_millis(1000);
    thread::sleep(ten_millis);

    let request = RpcRequest::GetBlock;
    let params = serde_json::json!([slot, {
    "maxSupportedTransactionVersion":0,
    }]);

    let block: EncodedConfirmedBlock = match rpc.send(request, params) {
        Ok(res) => res,
        Err(_) => return Err(AggregatorError::BlockFetchError),
    };
    handle_block(block, &mut database)
}

/// Processes a block of transactions and inserts them into the database.
///
/// # Arguments
///
/// * `block` - The encoded confirmed block containing transactions.
/// * `database` - The database instance.
///
/// # Errors
///
/// Returns an `AggregatorError` if there is an error fetching the block time or parsing a transaction.
fn handle_block(
    block: EncodedConfirmedBlock,
    database: &mut Database,
) -> Result<(), AggregatorError> {
    let transactions = &block.transactions;
    let block_time = match block.block_time {
        Some(res) => res,
        None => return Err(AggregatorError::TimeFetchError),
    };
    let time_stamp = get_timestamp(block_time);
    for encoded_transaction in transactions.iter() {
        let mut transaction = Transaction::new();
        transaction.timestamp = time_stamp.clone();
        match transaction.handle_transaction(encoded_transaction) {
            Ok(_) => transaction.insert_to_database(database),
            Err(_) => return Err(AggregatorError::TransactionParseError),
        };
    }

    Ok(())
}

/// Converts a Unix timestamp to a formatted string.
///
/// # Arguments
///
/// * `timestamp` - The Unix timestamp to convert.
///
/// # Returns
///
/// A string representing the formatted timestamp.
pub fn get_timestamp(timestamp: i64) -> String {
    let d = UNIX_EPOCH + Duration::from_secs(timestamp as u64);
    let datetime = DateTime::<Utc>::from(d);
    let timestamp_str = datetime.format("%Y-%m-%d %H:%M:%S").to_string();
    timestamp_str
}
