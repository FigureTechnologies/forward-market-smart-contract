#[cfg(test)]
mod execute_dealer_reset_tests {
    use crate::contract::execute;
    use crate::error::ContractError;
    use crate::msg::ExecuteMsg::DealerReset;
    use crate::storage::state_store::{
        save_buyer_state, save_contract_config, save_seller_state, Buyer, Config, Seller,
    };
    use cosmwasm_std::testing::{mock_env, mock_info};
    use cosmwasm_std::{to_json_binary, Addr, Attribute, Binary, ContractResult, SystemResult, Uint128, MessageInfo};
    use provwasm_mocks::mock_provenance_dependencies;
    use provwasm_std::types::cosmos::base::v1beta1::Coin;
    use provwasm_std::types::provenance::marker::v1::{
        Balance, QueryHoldingRequest, QueryHoldingResponse,
    };

    #[test]
    fn perform_dealer_reset() {
        let mut deps = mock_provenance_dependencies();
        let seller_address = deps.api.addr_make("allowed-seller-0");
        let buyer_address = deps.api.addr_make("contract-buyer");
        let dealer_address = deps.api.addr_make("dealer-address");
        let pool_denom = "test.token.asset.pool.0";
        let token_denom = "test.forward.market.token";
        let info = MessageInfo {
            sender: dealer_address.clone(),
            funds: vec![],
        };
        let env = mock_env();
        save_contract_config(
            &mut deps.storage,
            &Config {
                is_private: true,
                allowed_sellers: vec![seller_address.clone()],
                agreement_terms_hash: "mock-terms-hash".to_string(),
                token_denom: token_denom.into(),
                max_face_value_cents: Uint128::new(650000000),
                min_face_value_cents: Uint128::new(550000000),
                tick_size: Uint128::new(1000),
                dealers: vec![dealer_address.clone()],
                is_disabled: false,
            },
        )
        .unwrap();

        let pool_denoms = vec![pool_denom.into()];
        save_buyer_state(
            &mut deps.storage,
            &Buyer {
                buyer_address: buyer_address.clone(),
                has_accepted_pools: true,
            },
        )
        .unwrap();

        save_seller_state(
            &mut deps.storage,
            &Seller {
                seller_address: seller_address.clone(),
                accepted_value_cents: Uint128::new(550000000),
                pool_denoms,
                offer_hash: "mock-offer-hash".to_string(),
            },
        )
        .unwrap();

        let cb = Box::new(|bin: &Binary| -> SystemResult<ContractResult<Binary>> {
            let message = QueryHoldingRequest::try_from(bin.clone()).unwrap();

            let response = if message.id == "test.token.asset.pool.0" {
                let inner_deps = mock_provenance_dependencies();
                QueryHoldingResponse {
                    balances: vec![Balance {
                        address: inner_deps.api.addr_make("allowed-seller-0").to_string(),
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
                    seller_address: seller_address.clone(),
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
                let expected_buyer_state = Buyer {
                    buyer_address: Addr::unchecked(buyer_address),
                    has_accepted_pools: false,
                };
                let buyer_state_attr = response
                    .attributes
                    .clone()
                    .into_iter()
                    .find(|attr| -> bool { attr.key == "buyer_state" })
                    .unwrap();
                assert_eq!(
                    buyer_state_attr,
                    Attribute::new("buyer_state", format!("{:?}", expected_buyer_state))
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
        let buyer_address = deps.api.addr_make("contract-buyer");
        let dealer_address = deps.api.addr_make("dealer-address");
        let token_denom = "test.forward.market.token";
        let info = MessageInfo {
            sender: dealer_address.clone(),
            funds: vec![],
        };
        let env = mock_env();
        save_contract_config(
            &mut deps.storage,
            &Config {
                is_private: false,
                allowed_sellers: vec![],
                agreement_terms_hash: "mock-terms-hash".to_string(),
                token_denom: token_denom.into(),
                max_face_value_cents: Uint128::new(650000000),
                min_face_value_cents: Uint128::new(250000000),
                tick_size: Uint128::new(1000),
                dealers: vec![dealer_address.clone()],
                is_disabled: false,
            },
        )
        .unwrap();
        save_buyer_state(
            &mut deps.storage,
            &Buyer {
                buyer_address: buyer_address.clone(),
                has_accepted_pools: false,
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
        let seller_address = deps.api.addr_make("allowed-seller-0");
        let buyer_address = deps.api.addr_make("contract-buyer");
        let dealer_address = deps.api.addr_make("dealer-address");
        let pool_denom = "test.token.asset.pool.0";
        let token_denom = "test.forward.market.token";
        let info = MessageInfo {
            sender: seller_address.clone(),
            funds: vec![],
        };
        let env = mock_env();
        save_contract_config(
            &mut deps.storage,
            &Config {
                is_private: true,
                allowed_sellers: vec![seller_address.clone()],
                agreement_terms_hash: "mock-terms-hash".to_string(),
                token_denom: token_denom.into(),
                max_face_value_cents: Uint128::new(650000000),
                min_face_value_cents: Uint128::new(500000000),
                tick_size: Uint128::new(1000),
                dealers: vec![dealer_address.clone()],
                is_disabled: false,
            },
        )
        .unwrap();

        let pool_denoms = vec![pool_denom.into()];
        save_buyer_state(
            &mut deps.storage,
            &Buyer {
                buyer_address: buyer_address.clone(),
                has_accepted_pools: true,
            },
        )
        .unwrap();

        save_seller_state(
            &mut deps.storage,
            &Seller {
                seller_address: seller_address.clone(),
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
