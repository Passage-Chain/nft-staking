use cosmwasm_std::{attr, Event};
use std::vec;

use crate::contract::Config;

pub struct ConfigEvent<'a> {
    pub ty: &'a str,
    pub config: &'a Config,
}

impl<'a> From<ConfigEvent<'a>> for Event {
    fn from(ce: ConfigEvent) -> Self {
        Event::new(ce.ty.to_string()).add_attributes(vec![
            attr("vault_code_id", ce.config.vault_code_id.to_string()),
            attr("rewards_code_id", ce.config.rewards_code_id.to_string()),
        ])
    }
}

pub struct VaultEvent<'a> {
    pub ty: &'a str,
    pub address: &'a str,
}

impl<'a> From<VaultEvent<'a>> for Event {
    fn from(ve: VaultEvent) -> Self {
        Event::new(ve.ty.to_string()).add_attribute("address", ve.address.to_string())
    }
}
