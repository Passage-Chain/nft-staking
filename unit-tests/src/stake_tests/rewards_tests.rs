use cosmwasm_std::{coin, Addr, Uint128, Uint256};
use cw_multi_test::Executor;
use stardex_stake_rewards::state::UserReward;
use uju_cw2_common::error::CommonError;

use crate::{
    helpers::utils::assert_error,
    setup::{
        setup::{advance_blocks, setup, TextContext, BLOCK_TIME_NANOS},
        setup_accounts::{ALT_DENOM, INITIAL_BALANCE, NATIVE_DENOM},
    },
};

const REWARD_DURATION_SEC: u64 = 60 * 60 * 24 * 7 * 8;

#[test]
fn try_create_reward_account() {
    let TextContext {
        mut app,
        contracts,
        accounts,
    } = setup(0).unwrap();

    let admin = accounts.admin.clone();
    let user = accounts.users[0].clone();

    // Non admin cannot create reward account
    let fund_coin = coin(1_000_000_000, ALT_DENOM);
    let response = app.execute_contract(
        user.clone(),
        contracts.stake_native.clone(),
        &stardex_stake_native::contract::sv::ExecMsg::CreateRewardAccount {
            denom: ALT_DENOM.to_string(),
            duration_sec: REWARD_DURATION_SEC,
        },
        &[fund_coin],
    );
    assert_error(
        response,
        CommonError::Unauthorized("only the admin of contract can perform this action".to_string())
            .to_string(),
    );

    // Admin can create reward account
    let fund_coin = coin(1_000_000_000, ALT_DENOM);
    let response = app.execute_contract(
        admin.clone(),
        contracts.stake_native.clone(),
        &stardex_stake_native::contract::sv::ExecMsg::CreateRewardAccount {
            denom: ALT_DENOM.to_string(),
            duration_sec: REWARD_DURATION_SEC,
        },
        &[fund_coin],
    );
    assert!(response.is_ok());

    let reward_accounts = app
        .wrap()
        .query_wasm_smart::<Vec<Addr>>(
            contracts.stake_native.clone(),
            &stardex_stake_native::contract::sv::QueryMsg::RewardAccounts {},
        )
        .unwrap();
    assert_eq!(reward_accounts.len(), 1);
}

#[test]
fn try_earn_stake_rewards() {
    let TextContext {
        mut app,
        contracts,
        accounts,
    } = setup(0).unwrap();

    let admin = accounts.admin.clone();
    let user = accounts.users[0].clone();

    let fund_coin = coin(1_000_000_000, ALT_DENOM);
    let response = app.execute_contract(
        admin.clone(),
        contracts.stake_native.clone(),
        &stardex_stake_native::contract::sv::ExecMsg::CreateRewardAccount {
            denom: ALT_DENOM.to_string(),
            duration_sec: REWARD_DURATION_SEC,
        },
        &[fund_coin],
    );
    assert!(response.is_ok());

    // User stakes
    let stake_coin = coin(1_000_000, NATIVE_DENOM);
    let response = app.execute_contract(
        user.clone(),
        contracts.stake_native.clone(),
        &stardex_stake_native::contract::sv::ExecMsg::Stake { recipient: None },
        &[stake_coin],
    );
    assert!(response.is_ok());

    advance_blocks(&mut app, BLOCK_TIME_NANOS * 10);

    let balance_0 = app.wrap().query_balance(user.clone(), ALT_DENOM).unwrap();

    let response = app.execute_contract(
        user.clone(),
        contracts.stake_native.clone(),
        &stardex_stake_native::contract::sv::ExecMsg::ClaimRewards { recipient: None },
        &[],
    );
    assert!(response.is_ok());

    let balance_1 = app.wrap().query_balance(user.clone(), ALT_DENOM).unwrap();
    assert!(balance_0.amount < balance_1.amount);

    let reward_accounts = app
        .wrap()
        .query_wasm_smart::<Vec<Addr>>(
            contracts.stake_native.clone(),
            &stardex_stake_native::contract::sv::QueryMsg::RewardAccounts {},
        )
        .unwrap();
    let reward_account = reward_accounts[0].clone();

    let user_reward = app
        .wrap()
        .query_wasm_smart::<Option<UserReward>>(
            reward_account.clone(),
            &stardex_stake_rewards::contract::sv::QueryMsg::UserReward {
                address: user.to_string(),
            },
        )
        .unwrap()
        .unwrap();

    assert!(user_reward.rewards_checkpoint > Uint256::zero());
    assert_eq!(user_reward.pending_rewards, Uint128::zero());
    assert_eq!(
        user_reward.claimed_rewards,
        balance_1.amount - balance_0.amount
    );
}

#[test]
fn try_earn_stake_rewards_multiple_stakers() {
    let TextContext {
        mut app,
        contracts,
        accounts,
    } = setup(0).unwrap();

    let admin = accounts.admin.clone();
    let user_0 = accounts.users[0].clone();
    let user_1 = accounts.users[1].clone();

    let fund_coin = coin(1_000_000_000, ALT_DENOM);
    let response = app.execute_contract(
        admin.clone(),
        contracts.stake_native.clone(),
        &stardex_stake_native::contract::sv::ExecMsg::CreateRewardAccount {
            denom: ALT_DENOM.to_string(),
            duration_sec: REWARD_DURATION_SEC,
        },
        &[fund_coin],
    );
    assert!(response.is_ok());

    // User 0 stakes
    let stake_coin = coin(1_000_000, NATIVE_DENOM);
    let response = app.execute_contract(
        user_0.clone(),
        contracts.stake_native.clone(),
        &stardex_stake_native::contract::sv::ExecMsg::Stake { recipient: None },
        &[stake_coin],
    );
    assert!(response.is_ok());

    let half_duration_sec = REWARD_DURATION_SEC / 2;
    advance_blocks(&mut app, half_duration_sec * 1_000_000_000);

    // User 1 stakes
    let stake_coin = coin(1_000_000, NATIVE_DENOM);
    let response = app.execute_contract(
        user_1.clone(),
        contracts.stake_native.clone(),
        &stardex_stake_native::contract::sv::ExecMsg::Stake { recipient: None },
        &[stake_coin],
    );
    assert!(response.is_ok());

    advance_blocks(&mut app, half_duration_sec * 1_000_000_000);

    let response = app.execute_contract(
        user_0.clone(),
        contracts.stake_native.clone(),
        &stardex_stake_native::contract::sv::ExecMsg::ClaimRewards { recipient: None },
        &[],
    );
    assert!(response.is_ok());

    let response = app.execute_contract(
        user_1.clone(),
        contracts.stake_native.clone(),
        &stardex_stake_native::contract::sv::ExecMsg::ClaimRewards { recipient: None },
        &[],
    );
    assert!(response.is_ok());

    let user_0_balance = app.wrap().query_balance(user_0.clone(), ALT_DENOM).unwrap();
    let user_0_earned_tokens = user_0_balance.amount - Uint128::from(INITIAL_BALANCE);
    let user_1_balance = app.wrap().query_balance(user_1.clone(), ALT_DENOM).unwrap();
    let user_1_earned_tokens = user_1_balance.amount - Uint128::from(INITIAL_BALANCE);

    assert_eq!(
        user_0_earned_tokens,
        user_1_earned_tokens * Uint128::from(3u8)
    );
}
