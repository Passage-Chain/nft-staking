use cosmwasm_schema::write_api;
use vault_factory::contract::sv::{ContractExecMsg, ContractQueryMsg, InstantiateMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: ContractQueryMsg,
        execute: ContractExecMsg,
    }
}
