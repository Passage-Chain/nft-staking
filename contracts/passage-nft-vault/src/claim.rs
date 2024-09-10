use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, BlockInfo, CustomQuery, Deps, StdResult, Storage, Uint128};
use cw_storage_plus::{Map, Namespace};
use cw_utils::Expiration;

use crate::state::Nft;

#[cw_serde]
pub struct ClaimsResponse {
    pub claims: Vec<Claim>,
}

#[cw_serde]
pub struct Claim {
    pub nfts: Vec<Nft<Addr>>,
    pub release_at: Expiration,
}

impl Claim {
    pub fn new(nfts: Vec<Nft<Addr>>, released: Expiration) -> Self {
        Claim {
            nfts,
            release_at: released,
        }
    }
}

// TODO: revisit design (split each claim on own key?)
pub struct Claims(Map<&'static Addr, Vec<Claim>>);

impl Claims {
    pub const fn new(storage_key: &'static str) -> Self {
        Claims(Map::new(storage_key))
    }

    pub fn new_dyn(storage_key: impl Into<Namespace>) -> Self {
        Claims(Map::new_dyn(storage_key))
    }

    /// This creates a claim, such that the given address can claim an amount of tokens after
    /// the release date.
    pub fn create_claim(
        &self,
        storage: &mut dyn Storage,
        addr: &Addr,
        nfts: Vec<Nft<Addr>>,
        release_at: Expiration,
    ) -> StdResult<()> {
        // add a claim to this user to get their tokens after the unbonding period
        self.0.update(storage, addr, |old| -> StdResult<_> {
            let mut claims = old.unwrap_or_default();
            claims.push(Claim { nfts, release_at });
            Ok(claims)
        })?;
        Ok(())
    }

    /// This iterates over all mature claims for the address, and removes them, up to an optional cap.
    /// it removes the finished claims and returns the total amount of tokens to be released.
    pub fn claim_tokens(
        &self,
        storage: &mut dyn Storage,
        addr: &Addr,
        block: &BlockInfo,
        cap: Option<Uint128>,
    ) -> StdResult<Vec<Nft<Addr>>> {
        let mut to_send = vec![];
        self.0.update(storage, addr, |claim| -> StdResult<_> {
            let (_send, waiting): (Vec<_>, _) =
                claim.unwrap_or_default().into_iter().partition(|c| {
                    // if mature and we can pay fully, then include in _send
                    if c.release_at.is_expired(block) {
                        if let Some(limit) = cap {
                            if Uint128::from(to_send.len() as u64)
                                + Uint128::from(c.nfts.len() as u64)
                                > limit
                            {
                                return false;
                            }
                        }
                        to_send.extend(c.nfts.clone()); // Clone the nfts vector
                        true
                    } else {
                        // not to send, leave in waiting and save again
                        false
                    }
                });
            Ok(waiting)
        })?;
        Ok(to_send)
    }

    pub fn query_claims<Q: CustomQuery>(
        &self,
        deps: Deps<Q>,
        address: &Addr,
    ) -> StdResult<ClaimsResponse> {
        let claims = self.0.may_load(deps.storage, address)?.unwrap_or_default();
        Ok(ClaimsResponse { claims })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env},
        Order,
    };

    fn get_test_nfts(start: u64, end: u64) -> Vec<Nft<Addr>> {
        let mut retval: Vec<Nft<Addr>> = vec![];
        for i in start..end {
            retval.push(Nft {
                collection: Addr::unchecked(format!("collection-{}", i)),
                token_id: format!("tokenid-{}", i),
            })
        }
        retval
    }

    fn get_test_expiration() -> Expiration {
        Expiration::AtHeight(10)
    }

    #[test]
    fn can_create_claim() {
        let test_nfts = get_test_nfts(0, 3);
        let test_expiration = get_test_expiration();

        let claim = Claim::new(test_nfts.clone(), test_expiration);
        assert_eq!(claim.nfts, test_nfts);
        assert_eq!(claim.release_at, test_expiration);
    }

    #[test]
    fn can_create_claims() {
        let deps = mock_dependencies();
        let claims = Claims::new("claims");
        // Assert that claims creates a map and there are no keys in the map.
        assert_eq!(
            claims
                .0
                .range_raw(&deps.storage, None, None, Order::Ascending)
                .collect::<StdResult<Vec<_>>>()
                .unwrap()
                .len(),
            0
        );
    }

    #[test]
    fn check_create_claim_updates_map() {
        let test_nfts_0 = get_test_nfts(0, 3);
        let test_expiration = get_test_expiration();

        let mut deps = mock_dependencies();
        let claims = Claims::new("claims");

        claims
            .create_claim(
                deps.as_mut().storage,
                &Addr::unchecked("addr"),
                test_nfts_0.clone(),
                test_expiration,
            )
            .unwrap();

        // Assert that claims creates a map and there is one claim for the address.
        let saved_claims = claims
            .0
            .load(deps.as_mut().storage, &Addr::unchecked("addr"))
            .unwrap();
        assert_eq!(saved_claims.len(), 1);
        assert_eq!(saved_claims[0].nfts, test_nfts_0);
        assert_eq!(saved_claims[0].release_at, test_expiration);

        // Adding another claim to same address, make sure that both claims are saved.
        let test_nfts_1 = get_test_nfts(3, 5);
        claims
            .create_claim(
                deps.as_mut().storage,
                &Addr::unchecked("addr"),
                test_nfts_1.clone(),
                test_expiration,
            )
            .unwrap();

        // Assert that both claims exist for the address.
        let saved_claims = claims
            .0
            .load(deps.as_mut().storage, &Addr::unchecked("addr"))
            .unwrap();
        assert_eq!(saved_claims.len(), 2);
        assert_eq!(saved_claims[0].nfts, test_nfts_0);
        assert_eq!(saved_claims[0].release_at, test_expiration);
        assert_eq!(saved_claims[1].nfts, test_nfts_1);
        assert_eq!(saved_claims[1].release_at, test_expiration);

        // Adding another claim to different address, make sure that other address only has one claim.
        claims
            .create_claim(
                deps.as_mut().storage,
                &Addr::unchecked("addr2"),
                test_nfts_1.clone(),
                test_expiration,
            )
            .unwrap();

        // Assert that both claims exist for the address.
        let saved_claims = claims
            .0
            .load(deps.as_mut().storage, &Addr::unchecked("addr"))
            .unwrap();

        let saved_claims_addr2 = claims
            .0
            .load(deps.as_mut().storage, &Addr::unchecked("addr2"))
            .unwrap();
        assert_eq!(saved_claims.len(), 2);
        assert_eq!(saved_claims_addr2.len(), 1);
    }

    #[test]
    fn test_claim_tokens_with_no_claims() {
        let mut deps = mock_dependencies();
        let claims = Claims::new("claims");

        let claimable_nfts = claims
            .claim_tokens(
                deps.as_mut().storage,
                &Addr::unchecked("addr"),
                &mock_env().block,
                None,
            )
            .unwrap();
        let saved_claims = claims
            .0
            .load(deps.as_mut().storage, &Addr::unchecked("addr"))
            .unwrap();

        assert_eq!(claimable_nfts.len(), 0usize);
        assert_eq!(saved_claims.len(), 0);
    }

    #[test]
    fn test_claim_tokens_with_no_released_claims() {
        let mut deps = mock_dependencies();
        let claims = Claims::new("claims");

        let test_nfts_0 = get_test_nfts(0, 3);
        claims
            .create_claim(
                deps.as_mut().storage,
                &Addr::unchecked("addr"),
                test_nfts_0.clone(),
                Expiration::AtHeight(10),
            )
            .unwrap();

        let test_nfts_1 = get_test_nfts(3, 5);
        claims
            .create_claim(
                deps.as_mut().storage,
                &Addr::unchecked("addr"),
                test_nfts_1.clone(),
                Expiration::AtHeight(100),
            )
            .unwrap();

        let mut env = mock_env();
        env.block.height = 0;
        // the address has two claims however they are both not expired
        let claimable_nfts = claims
            .claim_tokens(
                deps.as_mut().storage,
                &Addr::unchecked("addr"),
                &env.block,
                None,
            )
            .unwrap();

        let saved_claims = claims
            .0
            .load(deps.as_mut().storage, &Addr::unchecked("addr"))
            .unwrap();

        assert_eq!(claimable_nfts.len(), 0usize);
        assert_eq!(saved_claims.len(), 2);
        assert_eq!(saved_claims[0].nfts, test_nfts_0);
        assert_eq!(saved_claims[0].release_at, Expiration::AtHeight(10));
        assert_eq!(saved_claims[1].nfts, test_nfts_1);
        assert_eq!(saved_claims[1].release_at, Expiration::AtHeight(100));
    }

    #[test]
    fn test_claim_tokens_with_one_released_claim() {
        let mut deps = mock_dependencies();
        let claims = Claims::new("claims");

        let test_nfts_0 = get_test_nfts(0, 3);
        claims
            .create_claim(
                deps.as_mut().storage,
                &Addr::unchecked("addr"),
                test_nfts_0.clone(),
                Expiration::AtHeight(10),
            )
            .unwrap();

        let test_nfts_1 = get_test_nfts(3, 5);
        claims
            .create_claim(
                deps.as_mut().storage,
                &Addr::unchecked("addr"),
                test_nfts_1.clone(),
                Expiration::AtHeight(100),
            )
            .unwrap();

        let mut env = mock_env();
        env.block.height = 20;
        // the address has two claims and the first one can be released
        let claimable_nfts = claims
            .claim_tokens(
                deps.as_mut().storage,
                &Addr::unchecked("addr"),
                &env.block,
                None,
            )
            .unwrap();

        let saved_claims = claims
            .0
            .load(deps.as_mut().storage, &Addr::unchecked("addr"))
            .unwrap();

        assert_eq!(claimable_nfts, test_nfts_0);
        assert_eq!(saved_claims.len(), 1);
        assert_eq!(saved_claims[0].nfts, test_nfts_1);
        assert_eq!(saved_claims[0].release_at, Expiration::AtHeight(100));
    }

    #[test]
    fn test_claim_tokens_with_all_released_claims() {
        let mut deps = mock_dependencies();
        let claims = Claims::new("claims");

        let test_nfts_0 = get_test_nfts(0, 3);
        claims
            .create_claim(
                deps.as_mut().storage,
                &Addr::unchecked("addr"),
                test_nfts_0.clone(),
                Expiration::AtHeight(10),
            )
            .unwrap();

        let test_nfts_1 = get_test_nfts(3, 5);
        claims
            .create_claim(
                deps.as_mut().storage,
                &Addr::unchecked("addr"),
                test_nfts_1.clone(),
                Expiration::AtHeight(100),
            )
            .unwrap();

        let mut env = mock_env();
        env.block.height = 1000;
        // the address has two claims and both can be released
        let claimable_nfts = claims
            .claim_tokens(
                deps.as_mut().storage,
                &Addr::unchecked("addr"),
                &env.block,
                None,
            )
            .unwrap();

        let saved_claims = claims
            .0
            .load(deps.as_mut().storage, &Addr::unchecked("addr"))
            .unwrap();

        let all_test_nfts: Vec<Nft<Addr>> = test_nfts_0
            .clone()
            .into_iter()
            .chain(test_nfts_1.clone().into_iter())
            .collect();
        assert_eq!(claimable_nfts, all_test_nfts);
        assert_eq!(saved_claims.len(), 0);
    }

    #[test]
    fn test_claim_tokens_with_zero_cap() {
        let mut deps = mock_dependencies();
        let claims = Claims::new("claims");

        let test_nfts_0 = get_test_nfts(0, 3);
        claims
            .create_claim(
                deps.as_mut().storage,
                &Addr::unchecked("addr"),
                test_nfts_0.clone(),
                Expiration::AtHeight(10),
            )
            .unwrap();

        let test_nfts_1 = get_test_nfts(3, 5);
        claims
            .create_claim(
                deps.as_mut().storage,
                &Addr::unchecked("addr"),
                test_nfts_1.clone(),
                Expiration::AtHeight(100),
            )
            .unwrap();

        let mut env = mock_env();
        env.block.height = 1000;

        let claimable_nfts = claims
            .claim_tokens(
                deps.as_mut().storage,
                &Addr::unchecked("addr"),
                &env.block,
                Some(Uint128::zero()),
            )
            .unwrap();

        let saved_claims = claims
            .0
            .load(deps.as_mut().storage, &Addr::unchecked("addr"))
            .unwrap();

        assert_eq!(claimable_nfts.len(), 0usize);
        assert_eq!(saved_claims.len(), 2);
        assert_eq!(saved_claims[0].nfts, test_nfts_0);
        assert_eq!(saved_claims[0].release_at, Expiration::AtHeight(10));
        assert_eq!(saved_claims[1].nfts, test_nfts_1);
        assert_eq!(saved_claims[1].release_at, Expiration::AtHeight(100));
    }

    #[test]
    fn test_claim_tokens_with_cap_greater_than_pending_claims() {
        let mut deps = mock_dependencies();
        let claims = Claims::new("claims");

        let test_nfts_0 = get_test_nfts(0, 3);
        claims
            .create_claim(
                deps.as_mut().storage,
                &Addr::unchecked("addr"),
                test_nfts_0.clone(),
                Expiration::AtHeight(10),
            )
            .unwrap();

        let test_nfts_1 = get_test_nfts(3, 5);
        claims
            .create_claim(
                deps.as_mut().storage,
                &Addr::unchecked("addr"),
                test_nfts_1.clone(),
                Expiration::AtHeight(100),
            )
            .unwrap();

        let mut env = mock_env();
        env.block.height = 1000;

        let claimable_nfts = claims
            .claim_tokens(
                deps.as_mut().storage,
                &Addr::unchecked("addr"),
                &env.block,
                Some(Uint128::from(6u64)),
            )
            .unwrap();

        let saved_claims = claims
            .0
            .load(deps.as_mut().storage, &Addr::unchecked("addr"))
            .unwrap();

        let all_test_nfts: Vec<Nft<Addr>> = test_nfts_0
            .clone()
            .into_iter()
            .chain(test_nfts_1.clone().into_iter())
            .collect();
        assert_eq!(claimable_nfts, all_test_nfts);
        assert_eq!(saved_claims.len(), 0);
    }

    #[test]
    fn test_claim_tokens_with_cap_only_one_claim_released() {
        let mut deps = mock_dependencies();
        let claims = Claims::new("claims");

        let test_nfts_0 = get_test_nfts(0, 3);
        claims
            .create_claim(
                deps.as_mut().storage,
                &Addr::unchecked("addr"),
                test_nfts_0.clone(),
                Expiration::AtHeight(10),
            )
            .unwrap();

        let test_nfts_1 = get_test_nfts(3, 5);
        claims
            .create_claim(
                deps.as_mut().storage,
                &Addr::unchecked("addr"),
                test_nfts_1.clone(),
                Expiration::AtHeight(5),
            )
            .unwrap();

        let mut env = mock_env();
        env.block.height = 1000;
        // the address has two claims and the first one can be released
        let claimable_nfts = claims
            .claim_tokens(
                deps.as_mut().storage,
                &Addr::unchecked("addr"),
                &env.block,
                Some(Uint128::from(2u64)),
            )
            .unwrap();
        assert_eq!(claimable_nfts, test_nfts_1);

        let saved_claims = claims
            .0
            .load(deps.as_mut().storage, &Addr::unchecked("addr"))
            .unwrap();
        assert_eq!(saved_claims.len(), 1);
        assert_eq!(saved_claims[0].nfts, test_nfts_0);
        assert_eq!(saved_claims[0].release_at, Expiration::AtHeight(10));
    }

    #[test]
    fn test_claim_tokens_with_cap_too_low_no_claims_released() {
        let mut deps = mock_dependencies();
        let claims = Claims::new("claims");

        let test_nfts_0 = get_test_nfts(0, 3);
        claims
            .create_claim(
                deps.as_mut().storage,
                &Addr::unchecked("addr"),
                test_nfts_0.clone(),
                Expiration::AtHeight(10),
            )
            .unwrap();

        let test_nfts_1 = get_test_nfts(3, 5);
        claims
            .create_claim(
                deps.as_mut().storage,
                &Addr::unchecked("addr"),
                test_nfts_1.clone(),
                Expiration::AtHeight(5),
            )
            .unwrap();

        let mut env = mock_env();
        env.block.height = 1000;
        // the address has two claims and the first one can be released
        let claimable_nfts = claims
            .claim_tokens(
                deps.as_mut().storage,
                &Addr::unchecked("addr"),
                &env.block,
                Some(Uint128::from(1u64)),
            )
            .unwrap();
        assert_eq!(claimable_nfts.len(), 0usize);

        let saved_claims = claims
            .0
            .load(deps.as_mut().storage, &Addr::unchecked("addr"))
            .unwrap();
        assert_eq!(saved_claims.len(), 2);
        assert_eq!(saved_claims[0].nfts, test_nfts_0);
        assert_eq!(saved_claims[0].release_at, Expiration::AtHeight(10));
        assert_eq!(saved_claims[1].nfts, test_nfts_1);
        assert_eq!(saved_claims[1].release_at, Expiration::AtHeight(5));
    }

    #[test]
    fn test_query_claims_returns_correct_claims() {
        let mut deps = mock_dependencies();
        let claims = Claims::new("claims");

        let test_nfts_0 = get_test_nfts(0, 3);

        claims
            .create_claim(
                deps.as_mut().storage,
                &Addr::unchecked("addr"),
                test_nfts_0.clone(),
                Expiration::AtHeight(10),
            )
            .unwrap();

        let queried_claims = claims
            .query_claims(deps.as_ref(), &Addr::unchecked("addr"))
            .unwrap();
        let saved_claims = claims
            .0
            .load(deps.as_mut().storage, &Addr::unchecked("addr"))
            .unwrap();
        assert_eq!(queried_claims.claims, saved_claims);
    }

    #[test]
    fn test_query_claims_returns_empty_for_non_existent_user() {
        let mut deps = mock_dependencies();
        let claims = Claims::new("claims");

        let test_nfts_0 = get_test_nfts(0, 3);

        claims
            .create_claim(
                deps.as_mut().storage,
                &Addr::unchecked("addr"),
                test_nfts_0.clone(),
                Expiration::AtHeight(10),
            )
            .unwrap();

        let queried_claims = claims
            .query_claims(deps.as_ref(), &Addr::unchecked("addr2"))
            .unwrap();

        assert_eq!(queried_claims.claims.len(), 0);
    }
}
