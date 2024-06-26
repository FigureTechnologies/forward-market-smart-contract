#[cfg(test)]
mod execute_dealer_reset_tests {
    use crate::contract::execute;
    use crate::error::ContractError;
    use crate::msg::ExecuteMsg::DealerReset;
    use crate::query::contract_state::query_contract_state;
    use crate::storage::state_store::{
        save_bid_list_state, save_contract_config, save_seller_state, Bid, BidList, Config, Seller,
    };
    use crate::version_info::{set_version_info, VersionInfoV1};
    use cosmwasm_std::testing::{mock_env, mock_info};
    use cosmwasm_std::{
        to_json_binary, Addr, Attribute, Binary, ContractResult, SystemResult, Uint128,
    };
    use provwasm_mocks::mock_provenance_dependencies;
    use provwasm_std::types::cosmos::base::v1beta1::Coin;
    use provwasm_std::types::provenance::marker::v1::{
        Balance, QueryHoldingRequest, QueryHoldingResponse,
    };

    #[test]
    fn perform_dealer_reset() {
        let mut deps = mock_provenance_dependencies();
        let seller_address = "allowed-seller-0";
        let buyer_address = "contract_buyer";
        let dealer_address = "dealer_address";
        let pool_denom = "test.token.asset.pool.0";
        let token_denom = "test.forward.market.token";
        let info = mock_info(dealer_address, &[]);
        let env = mock_env();
        save_contract_config(
            &mut deps.storage,
            &Config {
                use_private_sellers: true,
                use_private_buyers: false,
                allowed_sellers: vec![Addr::unchecked(seller_address)],
                allowed_buyers: vec![],
                token_denom: token_denom.into(),
                token_count: Uint128::new(1000),
                dealers: vec![Addr::unchecked(dealer_address)],
                is_disabled: false,
                max_bid_count: 2,
                contract_admin: Addr::unchecked("contract-admin"),
            },
        )
        .unwrap();

        set_version_info(
            &mut deps.storage,
            &VersionInfoV1 {
                definition: "mock".to_string(),
                version: "0.0.0".to_string(),
            },
        )
        .unwrap();

        let pool_denoms = vec![pool_denom.into()];

        save_bid_list_state(
            &mut deps.storage,
            &BidList {
                bids: vec![Bid {
                    buyer_address: Addr::unchecked(buyer_address),
                    agreement_terms_hash: "".to_string(),
                }],
            },
        )
        .unwrap();

        save_seller_state(
            &mut deps.storage,
            &Seller {
                seller_address: Addr::unchecked(seller_address),
                accepted_value_cents: Uint128::new(550000000),
                pool_denoms,
                offer_hash: "mock-offer-hash".to_string(),
            },
        )
        .unwrap();

        let cb = Box::new(|bin: &Binary| -> SystemResult<ContractResult<Binary>> {
            let message = QueryHoldingRequest::try_from(bin.clone()).unwrap();

            let response = if message.id == "test.token.asset.pool.0" {
                QueryHoldingResponse {
                    balances: vec![Balance {
                        address: seller_address.to_string(),
                        coins: vec![Coin {
                            denom: "test.token.asset.pool.0".to_string(),
                            amount: "1".to_string(),
                        }],
                    }],
                    pagination: None,
                }
            } else {
                panic!("unexpected query for denom")
            };

            let binary = to_json_binary(&response).unwrap();
            SystemResult::Ok(ContractResult::Ok(binary))
        });
        deps.querier
            .registered_custom_queries
            .insert("/provenance.marker.v1.Query/Holding".to_string(), cb);

        match execute(deps.as_mut(), env.clone(), info, DealerReset {}) {
            Ok(response) => {
                let expected_seller_state = Seller {
                    seller_address: Addr::unchecked(seller_address),
                    accepted_value_cents: Uint128::new(550000000),
                    pool_denoms: vec![],
                    offer_hash: "mock-offer-hash".to_string(),
                };
                let seller_state_attr = response
                    .attributes
                    .clone()
                    .into_iter()
                    .find(|attr| -> bool { attr.key == "seller_state" })
                    .unwrap();
                assert_eq!(
                    seller_state_attr,
                    Attribute::new("seller_state", format!("{:?}", expected_seller_state))
                );
                let expected_buyers = vec![Bid {
                    buyer_address: Addr::unchecked(buyer_address),
                    agreement_terms_hash: "".to_string(),
                }];
                assert_eq!(
                    query_contract_state(deps.as_ref()).unwrap().bids,
                    expected_buyers
                );
            }
            Err(error) => {
                panic!("failed to perform dealer reset: {:?}", error)
            }
        }
    }

    #[test]
    fn perform_dealer_reset_no_seller() {
        let mut deps = mock_provenance_dependencies();
        let buyer_address = "contract_buyer";
        let dealer_address = "dealer_address";
        let token_denom = "test.forward.market.token";
        let info = mock_info(dealer_address, &[]);
        let env = mock_env();
        save_contract_config(
            &mut deps.storage,
            &Config {
                use_private_sellers: false,
                use_private_buyers: false,
                allowed_sellers: vec![],
                allowed_buyers: vec![],
                token_denom: token_denom.into(),
                token_count: Uint128::new(1000),
                dealers: vec![Addr::unchecked(dealer_address)],
                is_disabled: false,
                max_bid_count: 1,
                contract_admin: Addr::unchecked("contract-admin"),
            },
        )
        .unwrap();

        save_bid_list_state(
            &mut deps.storage,
            &BidList {
                bids: vec![Bid {
                    buyer_address: Addr::unchecked(buyer_address),
                    agreement_terms_hash: "".to_string(),
                }],
            },
        )
        .unwrap();

        match execute(deps.as_mut(), env.clone(), info, DealerReset {}) {
            Ok(_) => {
                panic!("failed to detect error when resetting a contract without a seller")
            }
            Err(error) => {
                match error {
                    ContractError::InvalidDealerResetRequest => {}
                    _ => {
                        panic!("unexpected error encountered when resetting a contract without a seller")
                    }
                }
            }
        }
    }

    #[test]
    fn perform_dealer_reset_not_dealer() {
        let mut deps = mock_provenance_dependencies();
        let seller_address = "allowed-seller-0";
        let buyer_address = "contract_buyer";
        let dealer_address = "dealer_address";
        let pool_denom = "test.token.asset.pool.0";
        let token_denom = "test.forward.market.token";
        let info = mock_info(seller_address, &[]);
        let env = mock_env();
        save_contract_config(
            &mut deps.storage,
            &Config {
                use_private_sellers: true,
                use_private_buyers: false,
                allowed_sellers: vec![Addr::unchecked(seller_address)],
                allowed_buyers: vec![],
                token_denom: token_denom.into(),
                token_count: Uint128::new(1000),
                dealers: vec![Addr::unchecked(dealer_address)],
                is_disabled: false,
                max_bid_count: 6,
                contract_admin: Addr::unchecked("contract-admin"),
            },
        )
        .unwrap();

        save_bid_list_state(
            &mut deps.storage,
            &BidList {
                bids: vec![Bid {
                    buyer_address: Addr::unchecked(buyer_address),
                    agreement_terms_hash: "".to_string(),
                }],
            },
        )
        .unwrap();

        let pool_denoms = vec![pool_denom.into()];

        save_seller_state(
            &mut deps.storage,
            &Seller {
                seller_address: Addr::unchecked(seller_address),
                accepted_value_cents: Uint128::new(550000000),
                pool_denoms,
                offer_hash: "mock-offer-hash".to_string(),
            },
        )
        .unwrap();

        match execute(deps.as_mut(), env.clone(), info, DealerReset {}) {
            Ok(_) => {
                panic!("failed to detect error when resetting a contract as a non-dealer")
            }
            Err(error) => match error {
                ContractError::IllegalDealerResetRequest => {}
                _ => {
                    panic!("unexpected error encountered when resetting a contract as a non-dealer")
                }
            },
        }
    }
}
