#[derive(Debug)]
pub enum RuntimeError {
    AggregatorError(AggregatorError),
    WebServerError,
}

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
    InitTableError,
    InsertionError,
}
