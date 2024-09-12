use cosmwasm_schema::write_api;
use stake_rewards::contract::sv::{ContractExecMsg, ContractQueryMsg, InstantiateMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: ContractQueryMsg,
        execute: ContractExecMsg,
    }
}
