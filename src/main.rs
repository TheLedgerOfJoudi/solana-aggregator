use aggregator::aggregate_data;
use error::{AggregatorError, RuntimeError};
use std::thread;
mod aggregator;
mod database;
mod error;
mod restful_api;
mod tests;

/// The main entry point for the application.
///
/// This function starts two threads: one for running the web server and another
/// for running the data aggregation process. It waits for both threads to complete
/// and handles any errors that occur.
///
/// # Returns
///
/// A `Result` indicating the success or failure of the operation. Returns `Ok(())` if
/// both threads complete successfully, or a `RuntimeError` if an error occurs in either thread.
fn main() -> Result<(), RuntimeError> {
    let t1 = thread::spawn(restful_api::web_server);
    let t2 = thread::spawn(run);
    if t1.join().unwrap().is_err() {
        return Err(RuntimeError::WebServerError);
    } else if let Err(_err) = t2.join().unwrap() {
        return Err(RuntimeError::AggregatorError);
    };
    Ok(())
}

/// Runs the data aggregation process asynchronously.
///
/// This function initializes the data aggregation process by calling `aggregate_data()`.
/// It is designed to be run within a Tokio runtime.
///
/// # Returns
///
/// A `Result` indicating the success or failure of the data aggregation process. Returns `Ok(())`
/// if the aggregation completes successfully, or an `AggregatorError` if an error occurs.
#[tokio::main]
async fn run() -> Result<(), AggregatorError> {
    aggregate_data().await
}
