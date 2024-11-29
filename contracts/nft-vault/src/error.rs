use cosmwasm_std::{Instantiate2AddressError, OverflowError, StdError};
use cw_utils::PaymentError;
use thiserror::Error;
use uju_cw2_common::error::CommonError;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Instantiate2AddressError(#[from] Instantiate2AddressError),

    #[error("{0}")]
    PaymentError(#[from] PaymentError),
    #[error("{0}")]
    CommonError(#[from] CommonError),

    #[error("{0}")]
    OverflowError(#[from] OverflowError),

    #[error("RewardAccountNotFound")]
    RewardAccountNotFound,

    #[error("MaxClaimsReached")]
    MaxClaimsReached,

    #[error("ClaimableNftsNotFound")]
    ClaimableNftsNotFound,
}
