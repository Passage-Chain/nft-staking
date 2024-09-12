use cosmwasm_std::{attr, Event};
use std::vec;

use crate::state::{Config, CumulativeRewards, UserReward};

pub struct ConfigEvent<'a> {
    pub ty: &'a str,
    pub config: &'a Config,
}

impl<'a> From<ConfigEvent<'a>> for Event {
    fn from(ce: ConfigEvent) -> Self {
        Event::new(ce.ty.to_string()).add_attributes(vec![
            attr("stake", ce.config.stake.to_string()),
            attr("denom", ce.config.denom.to_string()),
            attr("duration_sec", ce.config.duration_sec.to_string()),
            attr("period_finish", ce.config.period_finish.to_string()),
            attr(
                "rewards_per_second",
                ce.config.rewards_per_second.to_string(),
            ),
        ])
    }
}

pub struct UpdateRewardsEvent<'a> {
    pub rewards: &'a CumulativeRewards,
}

impl<'a> From<UpdateRewardsEvent<'a>> for Event {
    fn from(ure: UpdateRewardsEvent) -> Self {
        Event::new("update-rewards".to_string()).add_attributes(vec![attr(
            "rewards_per_token",
            ure.rewards.rewards_per_token.to_string(),
        )])
    }
}

pub struct UpdateUserRewardsEvent<'a> {
    pub user_reward: &'a UserReward,
}

impl<'a> From<UpdateUserRewardsEvent<'a>> for Event {
    fn from(uure: UpdateUserRewardsEvent) -> Self {
        Event::new("update-user-rewards".to_string()).add_attributes(vec![
            attr(
                "rewards_checkpoint",
                uure.user_reward.rewards_checkpoint.to_string(),
            ),
            attr(
                "pending_rewards",
                uure.user_reward.pending_rewards.to_string(),
            ),
            attr(
                "claimed_rewards",
                uure.user_reward.claimed_rewards.to_string(),
            ),
        ])
    }
}
