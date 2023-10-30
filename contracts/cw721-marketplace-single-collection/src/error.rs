
use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Hash parse error: {0}")]
    ParseError(String),

    #[error("Invalid atomic swap id")]
    InvalidId {},

    #[error("Invalid preimage")]
    InvalidPreimage {},

    #[error("Invalid hash ({0} chars): must be 64 characters")]
    InvalidHash(usize),

    #[error("Send some coins to create an atomic swap")]
    EmptyBalance {},

    #[error("Must send exactly the required funds")]
    ExactFunds {},

    #[error("Invalid payment token")]
    InvalidPaymentToken {},

    #[error("Invalid input")]
    InvalidInput {},

    #[error("Insufficient contract balance")]
    InsufficientBalance {},
 
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Expired atomic swap")]
    Expired,
    #[error("Cancelled atomic swap")]
    Cancelled,
    #[error("Completed atomic swap")]
    Completed,
    #[error("Atomic swap already exists")]
    AlreadyExists,
}
