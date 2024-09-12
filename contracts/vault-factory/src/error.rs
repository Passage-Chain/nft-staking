use cosmwasm_std::{Instantiate2AddressError, StdError};
use thiserror::Error;
use uju_cw2_common::error::CommonError;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Instantiate2AddressError(#[from] Instantiate2AddressError),

    #[error("{0}")]
    CommonError(#[from] CommonError),
}
