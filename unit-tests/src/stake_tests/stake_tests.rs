use cosmwasm_std::{coin, Uint128};
use cw_multi_test::Executor;
use stardex_stake_native::error::ContractError;
use uju_cw2_common::error::CommonError;

use crate::{
    helpers::utils::assert_error,
    setup::{
        setup::{advance_blocks, setup, TextContext},
        setup_accounts::{ALT_DENOM, NATIVE_DENOM},
        setup_contracts::UNSTAKING_DURATION_SEC,
    },
};

#[test]
fn try_setup_stardex_stake_contracts() {
    let context = setup(0);
    assert!(context.is_ok());
}

#[test]
fn try_stardex_native_stake() {
    let TextContext {
        mut app,
        contracts,
        accounts,
    } = setup(0).unwrap();

    let user = accounts.users[0].clone();

    // Cannot stake invalid denom
    let stake_coin = coin(1_000_000, ALT_DENOM);
    let response = app.execute_contract(
        user.clone(),
        contracts.stake_native.clone(),
        &stardex_stake_native::contract::sv::ExecMsg::Stake { recipient: None },
        &[stake_coin],
    );
    assert_error(response, "Must send reserve token 'ustars'".to_owned());

    // Can stake valid denom
    let stake_coin = coin(1_000_000, NATIVE_DENOM);
    let response = app.execute_contract(
        user.clone(),
        contracts.stake_native.clone(),
        &stardex_stake_native::contract::sv::ExecMsg::Stake { recipient: None },
        &[stake_coin.clone()],
    );
    assert!(response.is_ok());

    let block_height = app.block_info().height;
    let stake_balance = app
        .wrap()
        .query_wasm_smart::<Uint128>(
            contracts.stake_native.clone(),
            &stardex_stake_native::contract::sv::QueryMsg::StakeBalanceAtHeight {
                address: user.to_string(),
                height: Some(block_height + 1),
            },
        )
        .unwrap();

    assert_eq!(stake_balance, stake_coin.amount);
}

#[test]
fn try_stardex_native_unstake() {
    let TextContext {
        mut app,
        contracts,
        accounts,
    } = setup(0).unwrap();

    let user = accounts.users[0].clone();
    let alt_user = accounts.users[1].clone();

    let stake_coin = coin(1_000_000, NATIVE_DENOM);
    let response = app.execute_contract(
        user.clone(),
        contracts.stake_native.clone(),
        &stardex_stake_native::contract::sv::ExecMsg::Stake { recipient: None },
        &[stake_coin.clone()],
    );
    assert!(response.is_ok());

    // Alt user cannot unstake
    let response = app.execute_contract(
        alt_user.clone(),
        contracts.stake_native.clone(),
        &stardex_stake_native::contract::sv::ExecMsg::Unstake {
            recipient: None,
            amount: stake_coin.amount,
        },
        &[],
    );
    assert_error(
        response,
        CommonError::InsufficientFunds("sender does not have enough staked balance".to_string())
            .to_string(),
    );

    // User cannot unstake more than they have staked
    let unstake_amount = stake_coin.amount + Uint128::from(1u64);
    let response = app.execute_contract(
        user.clone(),
        contracts.stake_native.clone(),
        &stardex_stake_native::contract::sv::ExecMsg::Unstake {
            recipient: None,
            amount: unstake_amount,
        },
        &[],
    );
    assert!(response.is_err());

    // User can unstake
    let response = app.execute_contract(
        user.clone(),
        contracts.stake_native.clone(),
        &stardex_stake_native::contract::sv::ExecMsg::Unstake {
            recipient: None,
            amount: stake_coin.amount,
        },
        &[],
    );
    assert!(response.is_ok());
}

#[test]
fn try_stardex_native_instant_claim() {
    let TextContext {
        mut app,
        contracts,
        accounts,
    } = setup(0).unwrap();

    let user = accounts.users[0].clone();

    let user_balance_1 = app
        .wrap()
        .query_balance(user.clone(), NATIVE_DENOM)
        .unwrap();

    let stake_coin = coin(1_000_000, NATIVE_DENOM);
    let response = app.execute_contract(
        user.clone(),
        contracts.stake_native.clone(),
        &stardex_stake_native::contract::sv::ExecMsg::Stake { recipient: None },
        &[stake_coin.clone()],
    );
    assert!(response.is_ok());

    let user_balance_2 = app
        .wrap()
        .query_balance(user.clone(), NATIVE_DENOM)
        .unwrap();
    assert_eq!(
        user_balance_1.amount - stake_coin.amount,
        user_balance_2.amount
    );

    let response = app.execute_contract(
        user.clone(),
        contracts.stake_native.clone(),
        &stardex_stake_native::contract::sv::ExecMsg::Unstake {
            recipient: None,
            amount: stake_coin.amount,
        },
        &[],
    );
    assert!(response.is_ok());

    let user_balance_3 = app
        .wrap()
        .query_balance(user.clone(), NATIVE_DENOM)
        .unwrap();
    assert_eq!(user_balance_2.amount, user_balance_3.amount);

    let response = app.execute_contract(
        user.clone(),
        contracts.stake_native.clone(),
        &stardex_stake_native::contract::sv::ExecMsg::Claim { recipient: None },
        &[],
    );
    assert!(response.is_ok());

    let user_balance_4 = app
        .wrap()
        .query_balance(user.clone(), NATIVE_DENOM)
        .unwrap();
    assert_eq!(user_balance_1.amount, user_balance_4.amount);
}

#[test]
fn try_stardex_native_delayed_claim() {
    let TextContext {
        mut app,
        contracts,
        accounts,
    } = setup(UNSTAKING_DURATION_SEC).unwrap();

    let user = accounts.users[0].clone();

    let user_balance_1 = app
        .wrap()
        .query_balance(user.clone(), NATIVE_DENOM)
        .unwrap();

    let stake_coin = coin(1_000_000, NATIVE_DENOM);
    let response = app.execute_contract(
        user.clone(),
        contracts.stake_native.clone(),
        &stardex_stake_native::contract::sv::ExecMsg::Stake { recipient: None },
        &[stake_coin.clone()],
    );
    assert!(response.is_ok());

    let user_balance_2 = app
        .wrap()
        .query_balance(user.clone(), NATIVE_DENOM)
        .unwrap();
    assert_eq!(
        user_balance_1.amount - stake_coin.amount,
        user_balance_2.amount
    );

    let response = app.execute_contract(
        user.clone(),
        contracts.stake_native.clone(),
        &stardex_stake_native::contract::sv::ExecMsg::Unstake {
            recipient: None,
            amount: stake_coin.amount,
        },
        &[],
    );
    assert!(response.is_ok());

    let user_balance_3 = app
        .wrap()
        .query_balance(user.clone(), NATIVE_DENOM)
        .unwrap();
    assert_eq!(user_balance_2.amount, user_balance_3.amount);

    // Cannot claim before unstaking period is over
    let response = app.execute_contract(
        user.clone(),
        contracts.stake_native.clone(),
        &stardex_stake_native::contract::sv::ExecMsg::Claim { recipient: None },
        &[],
    );
    assert_error(
        response,
        ContractError::ClaimableBalanceNotFound.to_string(),
    );

    advance_blocks(&mut app, UNSTAKING_DURATION_SEC * 1_000_000_000);

    let response = app.execute_contract(
        user.clone(),
        contracts.stake_native.clone(),
        &stardex_stake_native::contract::sv::ExecMsg::Claim { recipient: None },
        &[],
    );
    assert!(response.is_ok());

    let user_balance_4 = app
        .wrap()
        .query_balance(user.clone(), NATIVE_DENOM)
        .unwrap();
    assert_eq!(user_balance_1.amount, user_balance_4.amount);
}
