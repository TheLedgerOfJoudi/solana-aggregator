#[derive(Debug)]
pub enum RuntimeError {
    AggregatorError,
    WebServerError,
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, PartialEq)]
pub enum AggregatorError {
    BlockFetchError,
    EnvFetchError,
    PubsubClientError,
    SlotSubscribeError,
    MetaDataFetchError,
    TimeFetchError,
    TransactionParseError,
    DatabaseError,
}

#[derive(Debug)]
pub enum DatabaseError {
    ConnectError,
    InsertionError,
}
