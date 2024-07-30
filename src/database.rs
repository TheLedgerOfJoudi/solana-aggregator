use crate::error::DatabaseError;
use solana_sdk::pubkey::Pubkey;

use rusqlite::{Connection, Result};

/// Represents a database connection and provides methods for interacting with it.
pub struct Database {
    client: Connection,
}

impl Database {
    /// Creates a new `Database` instance with an initialized database connection.
    ///
    /// # Panics
    ///
    /// This function will panic if the database initialization fails.
    pub fn new() -> Database {
        let client = Database::init_database().unwrap();
        Database { client }
    }

    /// Establishes a new database connection.
    ///
    /// # Errors
    ///
    /// Returns `DatabaseError::ConnectError` if the connection to the database fails.
    pub fn new_connection() -> Result<Database, DatabaseError> {
        let client = match Connection::open("transactions.db") {
            Ok(res) => res,
            Err(_) => return Err(DatabaseError::ConnectError),
        };
        Ok(Database { client })
    }

    /// Initializes the database, creating the necessary tables if they do not exist.
    ///
    /// # Errors
    ///
    /// Returns `DatabaseError::ConnectError` if the connection to the database fails.
    /// Returns `DatabaseError::InitTableError` if the table creation fails.
    pub fn init_database() -> Result<Connection, DatabaseError> {
        let database_client = Connection::open("transactions.db").unwrap();

        database_client
            .execute(
                "
                CREATE TABLE IF NOT EXISTS transactions (
                    sender              text,
                    receiver            text,
                    amount              bigint,
                    timestamp           char(20),
                    signature           text
                    )
            ",
                [],
            )
            .unwrap();
        Ok(database_client)
    }

    /// Inserts a new transaction record into the database.
    ///
    /// # Arguments
    ///
    /// * `sender` - The sender's public key.
    /// * `receiver` - The receiver's public key.
    /// * `amount` - The transaction amount.
    /// * `timestamp` - The transaction timestamp.
    /// * `signature` - The transaction signature.
    ///
    /// # Errors
    ///
    /// Returns `DatabaseError::InsertionError` if the insertion fails.
    pub fn insert(
        &mut self,
        sender: Pubkey,
        receiver: Pubkey,
        amount: i64,
        timestamp: &String,
        signature: &String,
    ) -> Result<(), DatabaseError> {
        match self.client.execute(
            "INSERT INTO transactions (sender, receiver, amount, timestamp, signature) VALUES ($1, $2, $3, $4, $5)",
            [&sender.to_string(), &receiver.to_string(), &amount.to_string(), timestamp, signature],
        ){
            Ok(_) => Ok(()),
            Err(_) => Err(DatabaseError::InsertionError)
        }
    }

    /// Executes a query on the database and returns the results.
    ///
    /// # Arguments
    ///
    /// * `query` - The SQL query to execute.
    ///
    /// # Returns
    ///
    /// A vector of strings representing the query results.
    pub fn query(&mut self, query: &str) -> Vec<String> {
        let mut stmt = self.client.prepare(query).unwrap();
        let mut rows = stmt.query([]).unwrap();
        let mut query_response: Vec<String> = vec![];
        while let Ok(Some(row)) = rows.next() {
            let mut result = "{".to_string();
            if let Ok(res) = row.get::<usize, String>(0) {
                result.push_str("sender:");
                result.push_str(&res);
                result.push_str(", ");
            }

            if let Ok(res) = row.get::<usize, String>(1) {
                result.push_str("receiver:");
                result.push_str(&res);
                result.push_str(", ");
            }

            if let Ok(res) = row.get::<usize, i64>(2) {
                result.push_str("amount:");
                result.push_str(&res.to_string());
                result.push_str(", ");
            }

            if let Ok(res) = row.get::<usize, String>(3) {
                result.push_str("timestamp:");
                result.push_str(&res);
                result.push_str(", ");
            }

            if let Ok(res) = row.get::<usize, String>(4) {
                result.push_str("signature:");
                result.push_str(&res);
                result.push_str(", ");
            }
            result.push('}');
            query_response.push(result);
        }
        query_response
    }
}

impl Default for Database {
    fn default() -> Self {
        Self::new()
    }
}
