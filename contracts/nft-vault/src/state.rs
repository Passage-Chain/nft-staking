use std::fmt;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Api, StdError};
use cw_address_like::AddressLike;
use cw_storage_plus::{Index, IndexList, MultiIndex};

use crate::error::ContractError;

#[cw_serde]
pub struct Config<T: AddressLike> {
    pub rewards_code_id: u64,
    pub collections: Vec<T>,
    pub unstaking_duration_sec: u64,
}

impl Config<String> {
    pub fn str_to_addr(self, api: &dyn Api) -> Result<Config<Addr>, ContractError> {
        let mut collections = self
            .collections
            .into_iter()
            .map(|c| api.addr_validate(&c))
            .collect::<Result<Vec<Addr>, StdError>>()?;
        collections.sort();

        Ok(Config {
            rewards_code_id: self.rewards_code_id,
            collections,
            unstaking_duration_sec: self.unstaking_duration_sec,
        })
    }
}

#[cw_serde]
pub struct Nft<T: AddressLike> {
    pub collection: T,
    pub token_id: String,
}

impl fmt::Display for Nft<Addr> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.collection, self.token_id)
    }
}

impl Nft<String> {
    pub fn str_to_addr(self, api: &dyn Api) -> Result<Nft<Addr>, ContractError> {
        let collection = api.addr_validate(&self.collection)?;
        Ok(Nft {
            collection,
            token_id: self.token_id,
        })
    }
}

#[cw_serde]
pub struct StakedNft {
    pub staker: Addr,
    pub nft: Nft<Addr>,
}

// Collection, token_id
pub type StakedNftId = (Addr, String);

/// Defines indices for accessing staked NFTs
pub struct StakedNftIndices {
    // Index StakedNft by staker and collection
    pub staker_collection: MultiIndex<'static, (Addr, Addr), StakedNft, StakedNftId>,
}

impl<'a> IndexList<StakedNft> for StakedNftIndices {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<StakedNft>> + '_> {
        let v: Vec<&dyn Index<StakedNft>> = vec![&self.staker_collection];
        Box::new(v.into_iter())
    }
}
