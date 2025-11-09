use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ClientAccountError {
    #[error("Negative amount")]
    NegativeAmount,

    #[error("Insufficient available for withdrawal")]
    InsufficientBalance,

    #[error("Account is locked")]
    Locked,
}
