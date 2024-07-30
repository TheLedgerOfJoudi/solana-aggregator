#[allow(unused_imports)]
use crate::{aggregator, database::Database, error::AggregatorError};
#[allow(unused_imports)]
use std::env;

#[tokio::test]
async fn test_env() {
    env::set_var("rpc_url", "Invalid Url");
    env::set_var("wc_url", "Invalid Url");
    assert_eq!(
        Err(AggregatorError::EnvFetchError),
        aggregator::get_block(102000).await
    );
    assert_eq!(
        Err(AggregatorError::EnvFetchError),
        aggregator::aggregate_data().await
    );
    env::remove_var("rpc_url");
    env::remove_var("ws_url");
}

#[test]
fn test_get_timestamp() {
    let timestamp = 1722201110;
    assert_eq!("2024-07-28 21:11:50", aggregator::get_timestamp(timestamp));
}
