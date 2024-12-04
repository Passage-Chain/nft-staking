use cosmwasm_std::{
    coin, ensure, ensure_eq, to_json_binary, Addr, BankMsg, CosmosMsg, Response, StdResult,
    Timestamp, Uint128, Uint256, WasmMsg,
};
use cw2::set_contract_version;
use cw20::{BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg};
use cw_storage_plus::{Item, Map};
use cw_utils::{must_pay, nonpayable};
use sylvia::{
    contract,
    types::{ExecCtx, InstantiateCtx, QueryCtx},
};
use uju_cw2_common::error::CommonError;

use crate::state::RewardAsset;
use crate::{
    error::ContractError,
    events::{ConfigEvent, UpdateRewardsEvent, UpdateUserRewardsEvent},
    state::{Config, CumulativeRewards, UserReward},
};

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct StakeExternalRewardsContract {
    pub config: Item<Config>,
    pub rewards: Item<CumulativeRewards>,
    pub user_rewards: Map<Addr, UserReward>,
}

#[cfg(not(feature = "library"))]
use sylvia::entry_points;

#[cfg_attr(not(feature = "library"), entry_points)]
#[contract]
#[sv::error(ContractError)]
impl StakeExternalRewardsContract {
    pub const fn new() -> Self {
        Self {
            config: Item::new("C"),
            rewards: Item::new("R"),
            user_rewards: Map::new("U"),
        }
    }

    #[sv::msg(instantiate)]
    pub fn instantiate(
        &self,
        ctx: InstantiateCtx,
        stake: String,
        reward_asset: RewardAsset,
        period_start: Timestamp,
        duration_sec: u64,
    ) -> Result<Response, ContractError> {
        set_contract_version(ctx.deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

        ensure!(
            period_start >= ctx.env.block.time,
            CommonError::InvalidInput("period start must be at least current time".to_string())
        );

        let stake = ctx.deps.api.addr_validate(&stake)?;

        let fund_amount = match &reward_asset {
            RewardAsset::Native(denom) => must_pay(&ctx.info, &denom)?,
            RewardAsset::Cw20(cw20) => {
                let balance_response: BalanceResponse = ctx.deps.querier.query_wasm_smart(
                    cw20.to_string(),
                    &Cw20QueryMsg::Balance {
                        address: ctx.env.contract.address.to_string(),
                    },
                )?;
                balance_response.balance
            }
        };
        ensure!(
            fund_amount > Uint128::zero(),
            CommonError::InvalidInput("fund amount must be greater than zero".to_string())
        );

        let period_finish = Timestamp::from_seconds(ctx.env.block.time.seconds() + duration_sec);
        let rewards_per_second = fund_amount.checked_div(Uint128::from(duration_sec))?;

        ensure!(
            rewards_per_second > Uint128::zero(),
            CommonError::InvalidInput("reward rate must be greater than zero".to_string())
        );

        let config = &Config {
            stake,
            reward_asset,
            period_start,
            duration_sec,
            period_finish,
            rewards_per_second,
        };
        self.config.save(ctx.deps.storage, &config)?;

        self.rewards.save(
            ctx.deps.storage,
            &CumulativeRewards {
                rewards_per_token: Uint256::zero(),
                last_update: ctx.env.block.time,
            },
        )?;

        let response = Response::new().add_event(ConfigEvent {
            ty: "set-config",
            config: &config,
        });

        Ok(response)
    }

    #[sv::msg(exec)]
    pub fn stake_change(
        &self,
        mut ctx: ExecCtx,
        recipient: String,
        staked_amount: Uint128,
        total_staked: Uint128,
    ) -> Result<Response, ContractError> {
        nonpayable(&ctx.info)?;

        let config = self.config.load(ctx.deps.storage)?;
        ensure_eq!(
            config.stake,
            ctx.info.sender,
            CommonError::Unauthorized("sender is not the stake contract".to_string())
        );

        let rewards = self.update_rewards(&mut ctx, &config, total_staked)?;

        let recipient_addr = ctx.deps.api.addr_validate(&recipient)?;

        let current_user_reward = self
            .user_rewards
            .may_load(ctx.deps.storage, recipient_addr.clone())?
            .unwrap_or_default();

        let next_user_reward =
            current_user_reward.get_next_user_reward(rewards.rewards_per_token, staked_amount)?;

        self.user_rewards
            .save(ctx.deps.storage, recipient_addr, &next_user_reward)?;

        let response = Response::new()
            .add_event(UpdateRewardsEvent { rewards: &rewards })
            .add_event(UpdateUserRewardsEvent {
                user_reward: &next_user_reward,
            });

        Ok(response)
    }

    #[sv::msg(exec)]
    pub fn claim_rewards(
        &self,
        mut ctx: ExecCtx,
        recipient: String,
        staked_amount: Uint128,
        total_staked: Uint128,
    ) -> Result<Response, ContractError> {
        nonpayable(&ctx.info)?;

        let config = self.config.load(ctx.deps.storage)?;
        ensure_eq!(
            config.stake,
            ctx.info.sender,
            CommonError::Unauthorized("sender is not the stake contract".to_string())
        );

        let rewards = self.update_rewards(&mut ctx, &config, total_staked)?;

        let recipient_addr = ctx.deps.api.addr_validate(&recipient)?;

        let current_user_reward = self
            .user_rewards
            .may_load(ctx.deps.storage, recipient_addr.clone())?
            .unwrap_or_default();

        let mut next_user_reward =
            current_user_reward.get_next_user_reward(rewards.rewards_per_token, staked_amount)?;

        let send_msg: Option<CosmosMsg> = if next_user_reward.pending_rewards > Uint128::zero() {
            let claim_amount = next_user_reward.claim_rewards()?;

            match &config.reward_asset {
                RewardAsset::Native(denom) => Some(
                    BankMsg::Send {
                        to_address: recipient_addr.to_string(),
                        amount: vec![coin(claim_amount.u128(), denom)],
                    }
                    .into(),
                ),
                RewardAsset::Cw20(cw20) => Some(
                    WasmMsg::Execute {
                        contract_addr: cw20.to_string(),
                        msg: to_json_binary(&Cw20ExecuteMsg::Transfer {
                            recipient,
                            amount: claim_amount,
                        })?,
                        funds: vec![],
                    }
                    .into(),
                ),
            }
        } else {
            None
        };

        self.user_rewards
            .save(ctx.deps.storage, recipient_addr, &next_user_reward)?;

        let mut response = Response::new()
            .add_event(UpdateRewardsEvent { rewards: &rewards })
            .add_event(UpdateUserRewardsEvent {
                user_reward: &next_user_reward,
            });

        if let Some(send_msg) = send_msg {
            response = response.add_message(send_msg);
        }

        Ok(response)
    }

    #[sv::msg(query)]
    pub fn config(&self, ctx: QueryCtx) -> StdResult<Config> {
        self.config.load(ctx.deps.storage)
    }

    #[sv::msg(query)]
    pub fn rewards(&self, ctx: QueryCtx) -> StdResult<CumulativeRewards> {
        self.rewards.load(ctx.deps.storage)
    }

    #[sv::msg(query)]
    pub fn user_reward(&self, ctx: QueryCtx, address: String) -> StdResult<Option<UserReward>> {
        self.user_rewards
            .may_load(ctx.deps.storage, ctx.deps.api.addr_validate(&address)?)
    }

    #[sv::msg(query)]
    pub fn latest_user_reward(
        &self,
        ctx: QueryCtx,
        address: String,
        staked_amount: Uint128,
        total_staked: Uint128,
    ) -> Result<Option<UserReward>, ContractError> {
        let config = self.config.load(ctx.deps.storage)?;

        let rewards = self.rewards.load(ctx.deps.storage)?;
        let rewards_per_token = rewards.calc_rewards_per_token(&ctx.env, &config, total_staked)?;

        let current_user_reward = self
            .user_rewards
            .may_load(ctx.deps.storage, ctx.deps.api.addr_validate(&address)?)?
            .unwrap_or_default();

        let next_user_reward =
            current_user_reward.get_next_user_reward(rewards_per_token, staked_amount)?;

        Ok(Some(next_user_reward))
    }

    pub fn update_rewards(
        &self,
        ctx: &mut ExecCtx,
        config: &Config,
        total_staked: Uint128,
    ) -> Result<CumulativeRewards, ContractError> {
        let mut rewards = self.rewards.load(ctx.deps.storage)?;

        rewards.rewards_per_token =
            rewards.calc_rewards_per_token(&ctx.env, &config, total_staked)?;
        rewards.last_update = ctx.env.block.time;

        self.rewards.save(ctx.deps.storage, &rewards)?;

        Ok(rewards)
    }
}
