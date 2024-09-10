use cosmwasm_std::{StdResult, Timestamp};
use cw_multi_test::App;

use super::{
    setup_accounts::{setup_accounts, Accounts},
    setup_contracts::{setup_contracts, StardexContracts},
};

pub const GENESIS_MINT_START_TIME: u64 = 1647032400000000000;
pub const GENESIS_MINT_START_HEIGHT: u64 = 10000;
pub const BLOCK_TIME_NANOS: u64 = 5000000000;

pub struct TextContext {
    pub app: App,
    pub contracts: StardexContracts,
    pub accounts: Accounts,
}

pub fn set_block_time(app: &mut App, nanos: u64, height: Option<u64>) {
    let mut block = app.block_info();
    block.time = Timestamp::from_nanos(nanos);
    if let Some(h) = height {
        block.height = h;
    }
    app.set_block(block);
}

pub fn advance_blocks(app: &mut App, nanos: u64) {
    let block = app.block_info();
    let next_block_time = block.time.plus_nanos(nanos);
    let next_block_height = block.height + (nanos / BLOCK_TIME_NANOS);
    set_block_time(app, next_block_time.nanos(), Some(next_block_height));
}

pub fn setup(unstaking_duration_sec: u64) -> StdResult<TextContext> {
    let mut app = App::default();
    set_block_time(
        &mut app,
        GENESIS_MINT_START_TIME,
        Some(GENESIS_MINT_START_HEIGHT),
    );

    let accounts = setup_accounts(&mut app);
    let contracts = setup_contracts(&mut app, &accounts.admin, unstaking_duration_sec);

    Ok(TextContext {
        app,
        accounts,
        contracts,
    })
}
