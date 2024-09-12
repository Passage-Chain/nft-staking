use cosmwasm_std::{ConversionOverflowError, DivideByZeroError, OverflowError, StdError};
use cw_utils::PaymentError;
use thiserror::Error;
use uju_cw2_common::error::CommonError;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    PaymentError(#[from] PaymentError),

    #[error("{0}")]
    CommonError(#[from] CommonError),

    #[error("{0}")]
    DivideByZeroError(#[from] DivideByZeroError),

    #[error("{0}")]
    OverflowError(#[from] OverflowError),

    #[error("{0}")]
    ConversionOverflowError(#[from] ConversionOverflowError),

    #[error("UserRewardNotFound")]
    UserRewardNotFound,
}
