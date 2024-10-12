use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Env, Timestamp, Uint128, Uint256};
use std::{
    cmp::{max, min},
    convert::TryInto,
};

use crate::error::ContractError;

#[cw_serde]
pub struct Config {
    pub stake: Addr,
    pub denom: String,
    pub period_start: Timestamp,
    pub duration_sec: u64,
    pub period_finish: Timestamp,
    pub rewards_per_second: Uint128,
}

impl Config {
    pub fn first_reward_time(&self, last_update: Timestamp) -> Timestamp {
        max(self.period_start, last_update)
    }

    pub fn last_reward_time(&self, env: &Env) -> Timestamp {
        min(env.block.time, self.period_finish)
    }
}

#[cw_serde]
pub struct CumulativeRewards {
    pub rewards_per_token: Uint256,
    pub last_update: Timestamp,
}

impl CumulativeRewards {
    pub fn calc_rewards_per_token(
        &self,
        env: &Env,
        config: &Config,
        total_staked: Uint128,
    ) -> Result<Uint256, ContractError> {
        if total_staked == Uint128::zero() {
            return Ok(Uint256::zero());
        }

        let first_reward_time = config.first_reward_time(self.last_update);
        let last_reward_time = config.last_reward_time(env);
        let time_diff = last_reward_time.minus_seconds(first_reward_time.seconds());

        let additional_reward_per_token = config
            .rewards_per_second
            .full_mul(Uint128::from(time_diff.seconds()))
            .checked_mul(scale_factor())?
            .checked_div(Uint256::from(total_staked))?;

        Ok(self.rewards_per_token + additional_reward_per_token)
    }
}

#[cw_serde]
pub struct UserReward {
    pub rewards_checkpoint: Uint256,
    pub pending_rewards: Uint128,
    pub claimed_rewards: Uint128,
}

impl Default for UserReward {
    fn default() -> Self {
        UserReward {
            rewards_checkpoint: Uint256::zero(),
            pending_rewards: Uint128::zero(),
            claimed_rewards: Uint128::zero(),
        }
    }
}

impl UserReward {
    pub fn get_next_user_reward(
        &self,
        rewards_per_token: Uint256,
        stake_amount: Uint128,
    ) -> Result<Self, ContractError> {
        Ok(Self {
            rewards_checkpoint: rewards_per_token,
            pending_rewards: rewards_per_token
                .checked_sub(self.rewards_checkpoint)?
                .checked_mul(Uint256::from(stake_amount))?
                .checked_div(scale_factor())?
                .try_into()?,
            claimed_rewards: self.claimed_rewards,
        })
    }

    pub fn claim_rewards(&mut self) -> Result<Uint128, ContractError> {
        let claim_amount = self.pending_rewards;
        self.claimed_rewards = self.claimed_rewards.checked_add(claim_amount)?;
        self.pending_rewards = Uint128::zero();
        Ok(claim_amount)
    }
}

pub fn scale_factor() -> Uint256 {
    Uint256::from(10u8).pow(39)
}
