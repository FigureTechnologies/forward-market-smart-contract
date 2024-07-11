#[cfg(test)]
mod execute_disable_contract_tests {
    use crate::contract::execute;
    use crate::error::ContractError;
    use crate::msg::ExecuteMsg::{
        AcceptFinalizedPools, AddSeller, ContractDisable, DealerConfirm,
        FinalizePools, RescindFinalizedPools,
        UpdateAllowedSellers,
    };
    use crate::storage::state_store::{retrieve_contract_config, save_contract_config, save_seller_state, Config, Seller, save_bid_list_state, BidList, Bid};
    use cosmwasm_std::testing::{mock_env, mock_info};
    use cosmwasm_std::{Addr, Binary, ContractResult, SystemResult, to_json_binary, Uint128};
    use provwasm_mocks::mock_provenance_dependencies;
    use provwasm_std::types::cosmos::base::v1beta1::Coin;
    use provwasm_std::types::provenance::marker::v1::{Balance, QueryHoldingRequest, QueryHoldingResponse};
    use crate::query::contract_state::query_contract_state;
    use crate::version_info::{set_version_info, VersionInfoV1};

    #[test]
    fn execute_disable_contract() {
        let mut deps = mock_provenance_dependencies();
        let allowed_seller_address = "allowed-seller";
        let contract_admin = "contract-admin";
        let token_denom = "test.forward.market.token";
        let info = mock_info(contract_admin, &[]);
        let env = mock_env();
        let config = Config {
            use_private_sellers: true,
            use_private_buyers: false,
            allowed_sellers: vec![Addr::unchecked(allowed_seller_address)],
            allowed_buyers: vec![],
            dealers: vec![Addr::unchecked("dealer-address")],
            is_disabled: false,
            max_bid_count: 1,
            contract_admin: Addr::unchecked(contract_admin),
        };
        save_contract_config(&mut deps.storage, &config).unwrap();

        match execute(deps.as_mut(), env.clone(), info, ContractDisable {}) {
            Ok(_) => {
                let config_from_contract = retrieve_contract_config(&deps.storage).unwrap();
                let mut expected_config = config.clone();
                expected_config.is_disabled = true;
                assert_eq!(expected_config, config_from_contract);
            }
            Err(error) => {
                panic!("failed to disable contract: {:?}", error)
            }
        }
    }

    #[test]
    fn execute_disable_contract_unauthorized() {
        let mut deps = mock_provenance_dependencies();
        let allowed_seller_address = "allowed-seller";
        let token_denom = "test.forward.market.token";
        let info = mock_info(allowed_seller_address, &[]);
        let env = mock_env();
        let config = Config {
            use_private_sellers: true,
            use_private_buyers: false,
            allowed_sellers: vec![Addr::unchecked(allowed_seller_address)],
            allowed_buyers: vec![],
            dealers: vec![Addr::unchecked("dealer-address")],
            is_disabled: false,
            max_bid_count: 1,
            contract_admin: Addr::unchecked("contract-admin"),
        };
        save_contract_config(&mut deps.storage, &config).unwrap();

        match execute(deps.as_mut(), env.clone(), info, ContractDisable {}) {
            Ok(_) => {
                panic!(
                    "Failed to detect error when disabling the contract as neither the buyer \
                nor a dealer"
                )
            }
            Err(error) => match error {
                ContractError::UnauthorizedDisableRequest => {}
                _ => {
                    panic!(
                        "Unexpected error encountered when disabling a contract with an \
                        unauthorized party"
                    )
                }
            },
        }
    }

    #[test]
    fn execute_disable_contract_seller_already_finalized() {
        let mut deps = mock_provenance_dependencies();
        let allowed_seller_address = "allowed-seller";
        let contract_admin = "contract-admin";
        let token_denom = "test.forward.market.token";
        let info = mock_info(contract_admin, &[]);
        let env = mock_env();
        let config = Config {
            use_private_sellers: true,
            use_private_buyers: false,
            allowed_sellers: vec![Addr::unchecked(allowed_seller_address)],
            allowed_buyers: vec![],
            dealers: vec![Addr::unchecked("dealer-address")],
            is_disabled: false,
            max_bid_count: 5,
            contract_admin: Addr::unchecked(contract_admin),
        };
        save_contract_config(&mut deps.storage, &config).unwrap();

        save_seller_state(
            &mut deps.storage,
            &Seller {
                seller_address: Addr::unchecked(allowed_seller_address),
                accepted_value_cents: Uint128::new(450000000),
                pool_denoms: vec!["test.denom.pool.0".to_string()],
                offer_hash: "mock-offer-hash".to_string(),
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

        save_bid_list_state(
            &mut deps.storage,
            &BidList {
                bids: vec![Bid {
                    buyer_address: Addr::unchecked("buyer_address"),
                    agreement_terms_hash: "".to_string(),
                }],
            },
        )
            .unwrap();

        let cb = Box::new(|bin: &Binary| -> SystemResult<ContractResult<Binary>> {
            let message = QueryHoldingRequest::try_from(bin.clone()).unwrap();

            let response = if message.id == "test.denom.pool.0".to_string() {
                QueryHoldingResponse {
                    balances: vec![Balance {
                        address: allowed_seller_address.to_string(),
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

        match execute(deps.as_mut(), env.clone(), info, ContractDisable {}) {
            Ok(response) => {
                let expected_seller_state = Seller {
                    seller_address: Addr::unchecked(allowed_seller_address),
                    accepted_value_cents: Uint128::new(450000000),
                    pool_denoms: vec![],
                    offer_hash: "mock-offer-hash".to_string(),
                };
                assert_eq!(
                    query_contract_state(deps.as_ref()).unwrap().seller.unwrap(),
                    expected_seller_state
                );
            }
            Err(error) => {
                panic!("failed to perform dealer reset: {:?}", error)
            }
        }
    }

    #[test]
    fn disallow_all_executions_when_disabled() {
        let mut deps = mock_provenance_dependencies();
        let contract_admin = "contract-admin";
        let info = mock_info(contract_admin, &[]);
        let env = mock_env();
        save_contract_config(
            &mut deps.storage,
            &Config {
                use_private_sellers: true,
                use_private_buyers: false,
                allowed_sellers: vec![],
                allowed_buyers: vec![],
                dealers: vec![Addr::unchecked("dealer-address")],
                is_disabled: true,
                max_bid_count: 3,
                contract_admin: Addr::unchecked(contract_admin),
            },
        )
        .unwrap();

        [
            ContractDisable {},
            AddSeller {
                accepted_value_cents: Uint128::new(1),
                offer_hash: "mock-offer-hash".to_string(),
            },
            FinalizePools {
                pool_denoms: vec![],
            },
            DealerConfirm {},
            UpdateAllowedSellers {
                allowed_sellers: vec![],
            },
            AcceptFinalizedPools { offer_hash: "".to_string() },
            RescindFinalizedPools {},
        ]
        .into_iter()
        .for_each(|command| -> () {
            match execute(deps.as_mut(), env.clone(), info.clone(), command) {
                Ok(_) => {
                    panic!(
                        "Failed to detect error when attempting to execute against a \
                    disabled contract"
                    )
                }
                Err(error) => match error {
                    ContractError::InvalidContractExecution => {}
                    _ => {
                        panic!(
                            "Unexpected error encountered when attempting to execute \
                            against a disabled contract"
                        )
                    }
                },
            }
        });
    }
}
