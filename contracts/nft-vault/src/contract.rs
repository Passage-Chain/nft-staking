use cosmwasm_std::{
    attr, ensure, to_json_binary, Addr, Event, Response, StdResult, SubMsg, Timestamp, Uint128,
    WasmMsg,
};
use cw2::set_contract_version;
use cw_storage_plus::{IndexedMap, Item, Map, MultiIndex, SnapshotItem, Strategy};
use cw_utils::{maybe_addr, Expiration};
use stake_rewards::contract::sv::{
    ExecMsg as PassageRewardsExecuteMsg, InstantiateMsg as StakeRewardsInstantiateMsg,
};
use std::cmp::min;
use std::collections::HashMap;
use sylvia::types::{ExecCtx, InstantiateCtx, QueryCtx};
use sylvia::{contract, entry_points};
use uju_cw2_common::admin::only_contract_admin;
use uju_cw2_common::{
    address::address_or,
    error::CommonError,
    instantiate::{generate_instantiate_2_addr, generate_salt},
};
use uju_cw2_nft::helpers::{only_owner, transfer_nft};
use uju_index_query::{QueryOptions, QueryOptionsInternal};

use crate::{
    claim::{Claim, Claims},
    error::ContractError,
    events::{ConfigEvent, RewardAccountEvent},
    helpers::{setup_stake_change_messages, UpdateStakeResult},
    state::{Config, Nft, StakedNft, StakedNftId, StakedNftIndices},
};

pub const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const MAX_CLAIMS: usize = 100;
pub const MAX_NFTS: usize = 20;

pub struct NftVaultContract {
    pub config: Item<Config<Addr>>,
    pub reward_accounts: Item<Vec<Addr>>,
    pub users_staked_nfts: IndexedMap<StakedNftId, StakedNft, StakedNftIndices>,
    pub users_collection_staked_amounts: Map<(Addr, Addr), u64>,
    pub total_staked_amount: SnapshotItem<Uint128>,
    pub claims: Claims,
}

#[entry_points]
#[contract]
#[sv::error(ContractError)]
impl NftVaultContract {
    pub const fn new() -> Self {
        let indexes = StakedNftIndices {
            staker_collection: MultiIndex::new(
                |_pk: &[u8], s: &StakedNft| (s.staker.clone(), s.nft.collection.clone()),
                "n",
                "n_s",
            ),
        };

        Self {
            config: Item::new("C"),
            reward_accounts: Item::new("R"),
            users_staked_nfts: IndexedMap::new("n", indexes),
            users_collection_staked_amounts: Map::new("U"),
            total_staked_amount: SnapshotItem::new("t", "t_p", "t_l", Strategy::EveryBlock),
            claims: Claims::new("A"),
        }
    }

    #[sv::msg(instantiate)]
    pub fn instantiate(
        &self,
        ctx: InstantiateCtx,
        config: Config<String>,
    ) -> Result<Response, ContractError> {
        set_contract_version(ctx.deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

        let config = config.str_to_addr(ctx.deps.api)?;
        self.config.save(ctx.deps.storage, &config)?;

        self.reward_accounts.save(ctx.deps.storage, &vec![])?;

        self.total_staked_amount
            .save(ctx.deps.storage, &Uint128::zero(), ctx.env.block.height)?;

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
        rewards_code_id: Option<u64>,
        unstaking_duration_sec: Option<u64>,
    ) -> Result<Response, ContractError> {
        only_contract_admin(&ctx.deps.querier, &ctx.info, &ctx.env)?;

        let mut config = self.config.load(ctx.deps.storage)?;

        if let Some(rewards_code_id) = rewards_code_id {
            config.rewards_code_id = rewards_code_id;
        }

        if let Some(unstaking_duration_sec) = unstaking_duration_sec {
            config.unstaking_duration_sec = unstaking_duration_sec;
        }

        self.config.save(ctx.deps.storage, &config)?;

        let response = Response::new().add_event(ConfigEvent {
            ty: "update-config",
            config: &config,
        });

        Ok(response)
    }

    #[sv::msg(exec)]
    pub fn create_reward_account(
        &self,
        ctx: ExecCtx,
        label: String,
        denom: String,
        period_start: Timestamp,
        duration_sec: u64,
    ) -> Result<Response, ContractError> {
        only_contract_admin(&ctx.deps.querier, &ctx.info, &ctx.env)?;

        let config = self.config.load(ctx.deps.storage)?;
        let mut reward_accounts = self.reward_accounts.load(ctx.deps.storage)?;

        let salt = generate_salt(vec![
            ctx.env.contract.address.to_string().as_bytes(),
            (reward_accounts.len() as u64).to_be_bytes().as_ref(),
        ]);

        let reward_contract_addr = generate_instantiate_2_addr(
            &ctx.deps.as_ref(),
            &ctx.env.contract.address,
            config.rewards_code_id,
            &salt,
        )?;

        reward_accounts.push(reward_contract_addr.clone());
        self.reward_accounts
            .save(ctx.deps.storage, &reward_accounts)?;

        let instantiate_msg = WasmMsg::Instantiate2 {
            admin: Some(ctx.env.contract.address.to_string()),
            code_id: config.rewards_code_id,
            label: label.to_string(),
            msg: to_json_binary(&StakeRewardsInstantiateMsg {
                stake: ctx.env.contract.address.to_string(),
                denom,
                period_start,
                duration_sec,
            })?,
            funds: ctx.info.funds,
            salt,
        };

        let response = Response::new()
            .add_event(RewardAccountEvent {
                ty: "create-reward-account",
                address: &reward_contract_addr.to_string(),
            })
            .add_message(instantiate_msg);

        Ok(response)
    }

    #[sv::msg(exec)]
    pub fn stake(
        &self,
        ctx: ExecCtx,
        nfts: Vec<Nft<String>>,
        recipient: Option<String>,
    ) -> Result<Response, ContractError> {
        ensure!(
            !nfts.is_empty(),
            CommonError::InvalidInput("no nfts to stake".to_string())
        );
        ensure!(
            nfts.len() <= MAX_NFTS,
            CommonError::InvalidInput("too many nfts to stake".to_string())
        );

        let sender = address_or(
            &ctx.info.sender,
            maybe_addr(ctx.deps.api, recipient)?.as_ref(),
        );

        let config = self.config.load(ctx.deps.storage)?;
        let reward_accounts = self.reward_accounts.load(ctx.deps.storage)?;

        let mut response = Response::new();

        let mut collection_deltas: HashMap<Addr, i64> = HashMap::new();

        let internal_nfts = nfts
            .into_iter()
            .map(|nft| nft.str_to_addr(ctx.deps.api))
            .collect::<Result<Vec<Nft<cosmwasm_std::Addr>>, ContractError>>()?;

        for nft in internal_nfts {
            ensure!(
                &config.collections.contains(&nft.collection),
                CommonError::InvalidInput("collection not allowed".to_string())
            );

            // Update collection count
            let count = collection_deltas.entry(nft.collection.clone()).or_insert(0);
            *count = count.checked_add(1).unwrap();

            // Check owner and transfer NFT to contract
            only_owner(&ctx.deps.querier, &sender, &nft.collection, &nft.token_id)?;
            response = response.add_submessage(transfer_nft(
                &nft.collection,
                &nft.token_id,
                &ctx.env.contract.address,
            ));

            // Save staked NFT
            self.users_staked_nfts.save(
                ctx.deps.storage,
                (nft.collection.clone(), nft.token_id.clone()),
                &StakedNft {
                    staker: sender.clone(),
                    nft: nft,
                },
            )?;
        }

        let UpdateStakeResult {
            user_staked_amount,
            total_staked_amount,
        } = self.update_stake_amounts(ctx, config, &sender, collection_deltas)?;

        // Setup the stake change messages with the previous staked amount and total staked amount
        let stake_change_msgs = setup_stake_change_messages(
            &reward_accounts,
            &sender,
            user_staked_amount,
            total_staked_amount,
        )?;

        response = response.add_submessages(stake_change_msgs);

        Ok(response)
    }

    #[sv::msg(exec)]
    pub fn unstake(
        &self,
        ctx: ExecCtx,
        nfts: Vec<Nft<String>>,
        recipient: Option<String>,
    ) -> Result<Response, ContractError> {
        ensure!(
            !nfts.is_empty(),
            CommonError::InvalidInput("no nfts to stake".to_string())
        );
        ensure!(
            nfts.len() <= MAX_NFTS,
            CommonError::InvalidInput("too many nfts to unstake".to_string())
        );

        let sender = address_or(
            &ctx.info.sender,
            maybe_addr(ctx.deps.api, recipient)?.as_ref(),
        );

        let config = self.config.load(ctx.deps.storage)?;
        let reward_accounts = self.reward_accounts.load(ctx.deps.storage)?;

        let mut collection_deltas: HashMap<Addr, i64> = HashMap::new();

        let internal_nfts = nfts
            .into_iter()
            .map(|nft| nft.str_to_addr(ctx.deps.api))
            .collect::<Result<Vec<Nft<cosmwasm_std::Addr>>, ContractError>>()?;

        for nft in &internal_nfts {
            // Update collection count
            let count = collection_deltas.entry(nft.collection.clone()).or_insert(0);
            *count = count.checked_sub(1).unwrap();

            let staked_nft = self.users_staked_nfts.may_load(
                ctx.deps.storage,
                (nft.collection.clone(), nft.token_id.clone()),
            )?;
            ensure!(
                staked_nft.is_some(),
                CommonError::InvalidInput("nft not staked".to_string())
            );
            ensure!(
                staked_nft.unwrap().staker == sender,
                CommonError::Unauthorized("nft not staked by sender".to_string())
            );

            // Remove staked NFT
            self.users_staked_nfts.remove(
                ctx.deps.storage,
                (nft.collection.clone(), nft.token_id.clone()),
            )?;
        }

        // Create a claim for the unstaked nfts
        self.claims.create_claim(
            ctx.deps.storage,
            &sender,
            internal_nfts,
            Expiration::AtTime(
                ctx.env
                    .block
                    .time
                    .plus_seconds(config.unstaking_duration_sec),
            ),
        )?;

        let UpdateStakeResult {
            user_staked_amount,
            total_staked_amount,
        } = self.update_stake_amounts(ctx, config, &sender, collection_deltas)?;

        // Setup the stake change messages with the previous staked amount and total staked amount
        let stake_change_msgs = setup_stake_change_messages(
            &reward_accounts,
            &sender,
            user_staked_amount,
            total_staked_amount,
        )?;

        let response = Response::new().add_submessages(stake_change_msgs);

        Ok(response)
    }

    #[sv::msg(exec)]
    pub fn claim(
        &self,
        ctx: ExecCtx,
        recipient: Option<String>,
    ) -> Result<Response, ContractError> {
        let sender = address_or(
            &ctx.info.sender,
            maybe_addr(ctx.deps.api, recipient)?.as_ref(),
        );

        let claimable_nfts =
            self.claims
                .claim_tokens(ctx.deps.storage, &sender, &ctx.env.block, None)?;
        ensure!(
            !claimable_nfts.is_empty(),
            ContractError::ClaimableNftsNotFound
        );

        let mut response = Response::new();

        for nft in &claimable_nfts {
            response =
                response.add_submessage(transfer_nft(&nft.collection, &nft.token_id, &sender));
        }

        response = response.add_event(Event::new("claim-unstaked".to_string()).add_attributes(
            vec![
                    attr("sender", sender.to_string()),
                    attr(
                        "nfts",
                        claimable_nfts
                            .iter()
                            .map(|nft| nft.to_string())
                            .collect::<Vec<String>>()
                            .join(","),
                    ),
                ],
        ));

        Ok(response)
    }

    #[sv::msg(exec)]
    pub fn claim_rewards(
        &self,
        ctx: ExecCtx,
        recipient: Option<String>,
    ) -> Result<Response, ContractError> {
        let sender = address_or(
            &ctx.info.sender,
            maybe_addr(ctx.deps.api, recipient)?.as_ref(),
        );

        let config = self.config.load(ctx.deps.storage)?;
        let reward_accounts = self.reward_accounts.load(ctx.deps.storage)?;

        let empty_collection_deltas: HashMap<Addr, i64> = HashMap::new();
        let UpdateStakeResult {
            user_staked_amount,
            total_staked_amount,
        } = self.update_stake_amounts(ctx, config, &sender, empty_collection_deltas)?;

        let claim_json = to_json_binary(&PassageRewardsExecuteMsg::ClaimRewards {
            recipient: sender.to_string(),
            staked_amount: user_staked_amount,
            total_staked: total_staked_amount,
        })?;

        let claim_msgs = reward_accounts
            .iter()
            .map(|addr| {
                SubMsg::new(WasmMsg::Execute {
                    contract_addr: addr.to_string(),
                    msg: claim_json.clone(),
                    funds: vec![],
                })
            })
            .collect::<Vec<SubMsg>>();

        let response = Response::new()
            .add_event(
                Event::new("claim-rewards".to_string())
                    .add_attributes(vec![attr("sender", sender.to_string())]),
            )
            .add_submessages(claim_msgs);

        Ok(response)
    }

    #[sv::msg(query)]
    pub fn config(&self, ctx: QueryCtx) -> StdResult<Config<Addr>> {
        self.config.load(ctx.deps.storage)
    }

    #[sv::msg(query)]
    pub fn reward_accounts(&self, ctx: QueryCtx) -> StdResult<Vec<Addr>> {
        self.reward_accounts.load(ctx.deps.storage)
    }

    #[sv::msg(query)]
    pub fn users_staked_nfts(
        &self,
        ctx: QueryCtx,
        staker: String,
        query_options: QueryOptions<(String, String)>,
    ) -> StdResult<Vec<StakedNft>> {
        let staker = ctx.deps.api.addr_validate(&staker)?;

        let QueryOptionsInternal {
            limit,
            order,
            min,
            max,
        } = query_options.unpack(
            &|offset| {
                (
                    staker.clone(),
                    (
                        ctx.deps.api.addr_validate(&offset.0).unwrap(),
                        offset.1.to_string(),
                    ),
                )
            },
            None,
            None,
        );

        let results = self
            .users_staked_nfts
            .idx
            .staker_collection
            .sub_prefix(staker.clone())
            .range(ctx.deps.storage, min, max, order)
            .take(limit)
            .map(|res| res.map(|(_, pq)| pq))
            .collect::<StdResult<Vec<StakedNft>>>()?;

        Ok(results)
    }

    #[sv::msg(query)]
    pub fn users_collection_staked_amounts(
        &self,
        ctx: QueryCtx,
        staker: String,
        query_options: QueryOptions<String>,
    ) -> StdResult<Vec<(Addr, u64)>> {
        let staker = ctx.deps.api.addr_validate(&staker)?;

        let QueryOptionsInternal {
            limit,
            order,
            min,
            max,
        } = query_options.unpack(
            &|offset| ctx.deps.api.addr_validate(&offset).unwrap(),
            None,
            None,
        );

        let results = self
            .users_collection_staked_amounts
            .prefix(staker.clone())
            .range(ctx.deps.storage, min, max, order)
            .take(limit)
            .map(|res| res.map(|(collection, amount)| (collection, amount)))
            .collect::<StdResult<Vec<_>>>()?;

        Ok(results)
    }

    #[sv::msg(query)]
    pub fn total_staked_amount_at_height(
        &self,
        ctx: QueryCtx,
        height: Option<u64>,
    ) -> StdResult<Option<Uint128>> {
        let height = height.unwrap_or(ctx.env.block.height);

        let results = self
            .total_staked_amount
            .may_load_at_height(ctx.deps.storage, height)?;

        Ok(results)
    }

    #[sv::msg(query)]
    pub fn claims(&self, ctx: QueryCtx, staker: String) -> StdResult<Vec<Claim>> {
        let staker = ctx.deps.api.addr_validate(&staker)?;

        let results = self.claims.query_claims(ctx.deps, &staker)?;

        Ok(results.claims)
    }

    pub fn update_stake_amounts(
        &self,
        ctx: ExecCtx,
        config: Config<Addr>,
        sender: &Addr,
        collection_deltas: HashMap<Addr, i64>,
    ) -> Result<UpdateStakeResult, ContractError> {
        let mut user_staked_amount_before = u64::MAX;
        let mut user_staked_amount_after = u64::MAX;

        for collection in &config.collections {
            let user_collection_staked_amount_before = self
                .users_collection_staked_amounts
                .may_load(ctx.deps.storage, (sender.clone(), collection.clone()))?
                .unwrap_or_default();

            let user_collection_staked_amount_after = user_collection_staked_amount_before
                .checked_add_signed(*collection_deltas.get(collection).unwrap_or(&0i64))
                .unwrap();

            if user_collection_staked_amount_before != user_collection_staked_amount_after {
                self.users_collection_staked_amounts.save(
                    ctx.deps.storage,
                    (sender.clone(), collection.clone()),
                    &user_collection_staked_amount_after,
                )?;
            }

            user_staked_amount_before = min(
                user_staked_amount_before,
                user_collection_staked_amount_before,
            );

            user_staked_amount_after = min(
                user_staked_amount_after,
                user_collection_staked_amount_after,
            );
        }
        ensure!(
            user_staked_amount_before != u64::MAX,
            CommonError::InternalError("user staked amount before is u64::MAX".to_string())
        );
        ensure!(
            user_staked_amount_after != u64::MAX,
            CommonError::InternalError("user staked amount after is u64::MAX".to_string())
        );

        let total_staked_amount_before = self.total_staked_amount.load(ctx.deps.storage)?;
        let total_staked_amount_after = total_staked_amount_before
            .checked_sub(Uint128::from(user_staked_amount_before))?
            .checked_add(Uint128::from(user_staked_amount_after))?;

        if total_staked_amount_before != total_staked_amount_after {
            self.total_staked_amount.save(
                ctx.deps.storage,
                &total_staked_amount_after,
                ctx.env.block.height,
            )?;
        }

        Ok(UpdateStakeResult {
            user_staked_amount: Uint128::from(user_staked_amount_before),
            total_staked_amount: total_staked_amount_before,
        })
    }
}
