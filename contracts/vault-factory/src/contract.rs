use cosmwasm_schema::cw_serde;
use cosmwasm_std::{to_json_binary, Addr, Response, StdResult, WasmMsg};
use cw2::set_contract_version;
use cw_storage_plus::{Item, Map};
use nft_vault::{
    contract::sv::InstantiateMsg as NftVaultInstantiateMsg, state::Config as NftVaultConfig,
};
use sylvia::types::{ExecCtx, InstantiateCtx, QueryCtx};
use sylvia::{contract, entry_points};
use uju_cw2_common::{
    admin::only_contract_admin,
    instantiate::{generate_instantiate_2_addr, generate_salt},
};
use uju_index_query::{QueryOptions, QueryOptionsInternal};

use crate::{
    error::ContractError,
    events::{ConfigEvent, VaultEvent},
};

pub const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cw_serde]
pub struct Config {
    pub vault_code_id: u64,
    pub rewards_code_id: u64,
}

pub struct StakeVaultFactory {
    pub config: Item<Config>,
    pub vaults: Map<u64, Addr>,
}

#[entry_points]
#[contract]
#[sv::error(ContractError)]
impl StakeVaultFactory {
    pub const fn new() -> Self {
        Self {
            config: Item::new("C"),
            vaults: Map::new("N"),
        }
    }

    #[sv::msg(instantiate)]
    pub fn instantiate(
        &self,
        ctx: InstantiateCtx,
        vault_code_id: u64,
        rewards_code_id: u64,
    ) -> StdResult<Response> {
        set_contract_version(ctx.deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

        let config = &Config {
            vault_code_id,
            rewards_code_id,
        };
        self.config.save(ctx.deps.storage, &config)?;

        let response = Response::new().add_event(ConfigEvent {
            ty: "set-config",
            config: &config,
        });

        Ok(response)
    }

    #[sv::msg(exec)]
    pub fn update_config(
        &self,
        ctx: ExecCtx,
        vault_code_id: Option<u64>,
        rewards_code_id: Option<u64>,
    ) -> Result<Response, ContractError> {
        only_contract_admin(&ctx.deps.querier, &ctx.info, &ctx.env)?;

        let mut config = self.config.load(ctx.deps.storage)?;

        if let Some(vault_code_id) = vault_code_id {
            config.vault_code_id = vault_code_id;
        }

        if let Some(rewards_code_id) = rewards_code_id {
            config.rewards_code_id = rewards_code_id;
        }

        self.config.save(ctx.deps.storage, &config)?;

        let response = Response::new().add_event(ConfigEvent {
            ty: "update-config",
            config: &config,
        });

        Ok(response)
    }

    #[sv::msg(exec)]
    pub fn create_vault(
        &self,
        ctx: ExecCtx,
        vault_label: String,
        collections: Vec<String>,
        unstaking_duration_sec: u64,
    ) -> Result<Response, ContractError> {
        only_contract_admin(&ctx.deps.querier, &ctx.info, &ctx.env)?;

        let config = self.config.load(ctx.deps.storage)?;

        let last_vault_entry = self.vaults.last(ctx.deps.storage)?;
        let next_index = last_vault_entry.map_or(0u64, |(idx, _)| idx + 1);

        let salt = generate_salt(vec![
            ctx.env.contract.address.to_string().as_bytes(),
            next_index.to_be_bytes().as_ref(),
        ]);

        let vault_addr = generate_instantiate_2_addr(
            &ctx.deps.as_ref(),
            &ctx.env.contract.address,
            config.vault_code_id,
            &salt,
        )?;

        self.vaults
            .save(ctx.deps.storage, next_index, &vault_addr)?;

        let instantiate_msg = WasmMsg::Instantiate2 {
            admin: Some(ctx.info.sender.to_string()),
            code_id: config.vault_code_id,
            label: vault_label,
            msg: to_json_binary(&NftVaultInstantiateMsg {
                config: NftVaultConfig {
                    rewards_code_id: config.rewards_code_id,
                    collections,
                    unstaking_duration_sec,
                },
            })?,
            funds: ctx.info.funds,
            salt,
        };

        let response = Response::new()
            .add_event(VaultEvent {
                ty: "create-vault",
                address: &vault_addr.to_string(),
            })
            .add_message(instantiate_msg);

        Ok(response)
    }

    #[sv::msg(query)]
    pub fn vaults(
        &self,
        ctx: QueryCtx,
        query_options: QueryOptions<u64>,
    ) -> StdResult<Vec<(u64, Addr)>> {
        let QueryOptionsInternal {
            limit,
            order,
            min,
            max,
        } = query_options.unpack(&|&offset| offset, None, None);

        let results = self
            .vaults
            .range(ctx.deps.storage, min, max, order)
            .take(limit)
            .map(|res| res.map(|(collection, amount)| (collection, amount)))
            .collect::<StdResult<Vec<_>>>()?;

        Ok(results)
    }
}
