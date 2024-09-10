use cosmwasm_std::{attr, Addr, Event};
use std::vec;

use crate::state::Config;

pub struct ConfigEvent<'a> {
    pub ty: &'a str,
    pub config: &'a Config<Addr>,
}

impl<'a> From<ConfigEvent<'a>> for Event {
    fn from(ce: ConfigEvent) -> Self {
        Event::new(ce.ty.to_string()).add_attributes(vec![
            attr("rewards_code_id", ce.config.rewards_code_id.to_string()),
            attr(
                "collections",
                ce.config
                    .collections
                    .iter()
                    .map(|addr| addr.to_string())
                    .collect::<Vec<String>>()
                    .join(","),
            ),
            attr(
                "unstaking_duration_sec",
                ce.config.unstaking_duration_sec.to_string(),
            ),
        ])
    }
}

pub struct RewardAccountEvent<'a> {
    pub ty: &'a str,
    pub address: &'a str,
}

impl<'a> From<RewardAccountEvent<'a>> for Event {
    fn from(rae: RewardAccountEvent) -> Self {
        Event::new(rae.ty.to_string()).add_attribute("address", rae.address.to_string())
    }
}

pub struct StakeChangeEvent<'a> {
    pub ty: &'a str,
    pub sender: &'a str,
    pub amount: &'a str,
    pub total_staked: &'a str,
}

impl<'a> From<StakeChangeEvent<'a>> for Event {
    fn from(sce: StakeChangeEvent) -> Self {
        Event::new(sce.ty.to_string()).add_attributes(vec![
            attr("sender", sce.sender.to_string()),
            attr("amount", sce.amount.to_string()),
            attr("total_staked", sce.total_staked.to_string()),
        ])
    }
}
