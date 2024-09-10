use cosmwasm_std::Addr;
use cw_multi_test::{App, ContractWrapper, Executor};

use super::setup_accounts::NATIVE_DENOM;

pub const UNSTAKING_DURATION_SEC: u64 = 60 * 60 * 24 * 7;

pub struct StardexContracts {
    pub stake_native: Addr,
}

pub fn setup_contracts(
    app: &mut App,
    admin: &Addr,
    unstaking_duration_sec: u64,
) -> StardexContracts {
    let stake_rewards_code = ContractWrapper::new(
        stardex_stake_rewards::contract::entry_points::execute,
        stardex_stake_rewards::contract::entry_points::instantiate,
        stardex_stake_rewards::contract::entry_points::query,
    );
    let stake_rewards_code_id = app.store_code(Box::new(stake_rewards_code));

    let stake_native_code = ContractWrapper::new(
        stardex_stake_native::contract::entry_points::execute,
        stardex_stake_native::contract::entry_points::instantiate,
        stardex_stake_native::contract::entry_points::query,
    );
    let stake_native_code_id = app.store_code(Box::new(stake_native_code));
    let stake_native_addr = app
        .instantiate_contract(
            stake_native_code_id,
            admin.clone(),
            &stardex_stake_native::contract::sv::InstantiateMsg {
                rewards_code_id: stake_rewards_code_id,
                denom: NATIVE_DENOM.to_string(),
                unstaking_duration_sec,
            },
            &[],
            "Stardex Stake Native",
            Some(admin.to_string()),
        )
        .unwrap();

    StardexContracts {
        stake_native: stake_native_addr,
    }
}
