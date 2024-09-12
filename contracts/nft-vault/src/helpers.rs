use cosmwasm_std::{to_json_binary, Addr, SubMsg, Uint128, WasmMsg};
use stake_rewards::contract::sv::ExecMsg as PassageRewardsExecuteMsg;

use crate::error::ContractError;

pub struct UpdateStakeResult {
    pub user_staked_amount: Uint128,
    pub total_staked_amount: Uint128,
}

pub fn setup_stake_change_messages(
    reward_accounts: &[Addr],
    sender: &Addr,
    staked_amount: Uint128,
    total_staked: Uint128,
) -> Result<Vec<SubMsg>, ContractError> {
    let stake_json = to_json_binary(&PassageRewardsExecuteMsg::StakeChange {
        recipient: sender.to_string(),
        staked_amount,
        total_staked,
    })?;

    let sub_msgs = reward_accounts
        .iter()
        .map(|addr| {
            SubMsg::new(WasmMsg::Execute {
                contract_addr: addr.to_string(),
                msg: stake_json.clone(),
                funds: vec![],
            })
        })
        .collect::<Vec<SubMsg>>();

    Ok(sub_msgs)
}
