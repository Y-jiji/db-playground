#[derive(Debug)]
pub enum MThreadServiceError<T> {
    // shutdown error if there is any
    ShutdownErrorReport(Vec<Box<dyn std::any::Any + Send>>),
    /// query errors
    TxAborted,
    TxPending,
    /// return the transaction back if cannot send a transaction to service
    SendError(T),
    /// no sender is open
    NoSenderOpen,
}
