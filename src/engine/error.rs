use thiserror::Error;

use crate::{
    client::error::ClientAccountError,
    types::{ClientId, TransactionId},
};

#[derive(Error, Debug, PartialEq)]
pub enum EngineError {
    #[error("Client not found")]
    ClientNotFound,

    #[error("Client account error: {0}")]
    ClientAccountError(#[from] ClientAccountError),

    #[error("InvalidLedger: {0}")]
    InvalidLeger(TransactionId),

    #[error("Transaction not found: {0}")]
    TransactionNotFound(TransactionId),

    #[error("Transaction disputed already: {0}")]
    TransactionAlreadyDisputed(TransactionId),

    #[error("Transaction not disputed: {0}")]
    TransactionNotDisputed(TransactionId),

    #[error("Transaction with ID '{0}' is not owned by the client {1}")]
    NotClientOwnedTransaction(TransactionId, ClientId),

    #[error("Transaction already exists")]
    TransactionAlreadyExists,

    #[error("Error writing console")]
    WriteBuffer,
}
