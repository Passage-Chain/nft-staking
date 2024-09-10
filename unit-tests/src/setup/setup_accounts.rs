use cosmwasm_std::{coin, Addr, Coin};
use cw_multi_test::App;
use cw_multi_test::{BankSudo, SudoMsg};

pub const INITIAL_BALANCE: u128 = 500_000_000_000;
pub const NATIVE_DENOM: &str = "ustars";
pub const ALT_DENOM: &str = "uatom";
pub const NUM_USERS: u128 = 4;

pub struct Accounts {
    pub admin: Addr,
    pub users: Vec<Addr>,
}

// initializes accounts with balances
pub fn setup_accounts(app: &mut App) -> Accounts {
    let mut accounts = Accounts {
        admin: app.api().addr_make("admin"),
        users: vec![],
    };

    let funds: Vec<Coin> = vec![
        coin(INITIAL_BALANCE, NATIVE_DENOM),
        coin(INITIAL_BALANCE, ALT_DENOM),
    ];

    app.sudo(SudoMsg::Bank({
        BankSudo::Mint {
            to_address: accounts.admin.to_string(),
            amount: funds.clone(),
        }
    }))
    .map_err(|err| println!("{:?}", err))
    .ok();

    for idx in 0..NUM_USERS {
        let user = app.api().addr_make(&format!("user-{}", idx));

        app.sudo(SudoMsg::Bank({
            BankSudo::Mint {
                to_address: user.to_string(),
                amount: funds.clone(),
            }
        }))
        .map_err(|err| println!("{:?}", err))
        .ok();

        accounts.users.push(user);
    }

    accounts
}
