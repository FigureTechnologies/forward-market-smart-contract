#[cfg(test)]
mod execute_finalize_pools_tests {
    use crate::contract::execute;
    use crate::error::ContractError;
    use crate::msg::ExecuteMsg::FinalizePools;
    use crate::storage::state_store::{save_contract_config, save_seller_state, Config, Seller};
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
    fn execute_finalize_pool() {
        let mut deps = mock_provenance_dependencies();
        let seller_address = "allowed-seller-0";
        let pool_denom = "test.token.asset.pool.0";
        let token_denom = "test.forward.market.token";
        let info = mock_info(seller_address, &[]);
        let env = mock_env();
        save_contract_config(
            &mut deps.storage,
            &Config {
                use_private_sellers: true,
                use_private_buyers: true,
                allowed_sellers: vec![Addr::unchecked(seller_address)],
                allowed_buyers: vec![],
                token_denom: token_denom.into(),
                max_face_value_cents: Uint128::new(650000000),
                min_face_value_cents: Uint128::new(350000000),
                tick_size: Uint128::new(1000),
                dealers: vec![Addr::unchecked("dealer-address")],
                is_disabled: false,
                max_bid_count: 1,
                contract_admin: Addr::unchecked("contract-admin")
            },
        )
        .unwrap();

        let pool_denoms = vec![pool_denom.into()];
        save_seller_state(
            &mut deps.storage,
            &Seller {
                seller_address: Addr::unchecked(seller_address),
                accepted_value_cents: Uint128::new(650000000),
                pool_denoms: vec![],
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
                            amount: "2".to_string(),
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

        match execute(
            deps.as_mut(),
            env.clone(),
            info,
            FinalizePools { pool_denoms },
        ) {
            Ok(response) => {
                let expected_seller_state = Seller {
                    seller_address: Addr::unchecked(seller_address),
                    accepted_value_cents: Uint128::new(650000000),
                    pool_denoms: vec![pool_denom.into()],
                    offer_hash: "mock-offer-hash".to_string(),
                };
                assert_eq!(response.attributes.len(), 1);
                assert_eq!(
                    response.attributes[0],
                    Attribute::new("seller_state", format!("{:?}", expected_seller_state))
                );
            }
            Err(error) => {
                panic!("failed to finalize the list pool denoms: {:?}", error)
            }
        }
    }

    #[test]
    fn execute_finalize_pool_invalid_list() {
        let mut deps = mock_provenance_dependencies();
        let seller_address = "allowed-seller-0";
        let token_denom = "test.forward.market.token";
        let info = mock_info(seller_address, &[]);
        let env = mock_env();

        save_contract_config(
            &mut deps.storage,
            &Config {
                use_private_sellers: true,
                use_private_buyers: true,
                allowed_sellers: vec![Addr::unchecked(seller_address)],
                allowed_buyers: vec![],
                token_denom: token_denom.into(),
                max_face_value_cents: Uint128::new(650000000),
                min_face_value_cents: Uint128::new(450000000),
                tick_size: Uint128::new(1000),
                dealers: vec![Addr::unchecked("dealer-address")],
                is_disabled: false,
                max_bid_count: 2,
                contract_admin: Addr::unchecked("contract-admin")
            },
        )
        .unwrap();

        save_seller_state(
            &mut deps.storage,
            &Seller {
                seller_address: Addr::unchecked(seller_address),
                accepted_value_cents: Uint128::new(650000000),
                pool_denoms: vec![],
                offer_hash: "mock-offer-hash".to_string(),
            },
        )
        .unwrap();

        match execute(
            deps.as_mut(),
            env.clone(),
            info,
            FinalizePools {
                pool_denoms: vec![],
            },
        ) {
            Ok(_) => {
                panic!("failed to detect error when finalizing an empty list of pool denoms")
            }
            Err(error) => match error {
                ContractError::InvalidFinalizationRequest => {}
                _ => {
                    panic!(
                        "an unexpected error was returned when attempting to finalize an empty \
                            list of pool denoms"
                    )
                }
            },
        }
    }

    #[test]
    fn execute_finalize_pool_not_seller() {
        let mut deps = mock_provenance_dependencies();
        let unauthorized_seller_address = "unauthorized-seller";
        let allowed_seller_address = "allowed-seller";
        let token_denom = "test.forward.market.token";
        let info = mock_info(unauthorized_seller_address, &[]);
        let env = mock_env();
        save_contract_config(
            &mut deps.storage,
            &Config {
                use_private_sellers: true,
                use_private_buyers: true,
                allowed_sellers: vec![Addr::unchecked(allowed_seller_address)],
                allowed_buyers: vec![],
                token_denom: token_denom.into(),
                max_face_value_cents: Uint128::new(550000000),
                min_face_value_cents: Uint128::new(550000000),
                tick_size: Uint128::new(1000),
                dealers: vec![Addr::unchecked("dealer-address")],
                is_disabled: false,
                max_bid_count: 2,
                contract_admin: Addr::unchecked("contract-admin")
            },
        )
        .unwrap();

        save_seller_state(
            &mut deps.storage,
            &Seller {
                seller_address: Addr::unchecked(allowed_seller_address),
                accepted_value_cents: Uint128::new(650000000),
                pool_denoms: vec![],
                offer_hash: "mock-offer-hash".to_string(),
            },
        )
        .unwrap();

        match execute(
            deps.as_mut(),
            env.clone(),
            info,
            FinalizePools {
                pool_denoms: vec!["test.denom.0".into()],
            },
        ) {
            Ok(_) => {
                panic!("failed to detect error when finalizing with an invalid seller")
            }
            Err(error) => match error {
                ContractError::UnauthorizedAsSeller => {}
                _ => {
                    panic!(
                        "an unexpected error was returned when attempting to finalize a list \
                            of pools with an address that does not belong to the seller"
                    )
                }
            },
        }
    }
}
