use cosmwasm_schema::write_api;
use passage_nft_vault::contract::sv::{ContractExecMsg, ContractQueryMsg, InstantiateMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: ContractQueryMsg,
        execute: ContractExecMsg,
    }
}
