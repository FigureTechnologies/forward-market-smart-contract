#[cfg(test)]
mod execute_disable_contract_tests {
    use crate::contract::execute;
    use crate::error::ContractError;
    use crate::msg::ExecuteMsg::{
        AcceptFinalizedPools, AddSeller, ContractDisable, DealerConfirm, DealerReset,
        FinalizePools, RemoveAsSeller, RescindFinalizedPools, UpdateAgreementTermsHash,
        UpdateAllowedSellers, UpdateFaceValueCents,
    };
    use crate::storage::state_store::{
        retrieve_contract_config, save_buyer_state, save_contract_config, save_seller_state, Buyer,
        Config, Seller,
    };
    use cosmwasm_std::testing::{mock_env, mock_info};
    use cosmwasm_std::{Addr, Uint128};
    use provwasm_mocks::mock_provenance_dependencies;

    #[test]
    fn execute_disable_contract() {
        let mut deps = mock_provenance_dependencies();
        let allowed_seller_address = "allowed-seller";
        let buyer_address = "buyer-address";
        let token_denom = "test.forward.market.token";
        let info = mock_info(buyer_address, &[]);
        let env = mock_env();
        let config = Config {
            is_private: true,
            allowed_sellers: vec![Addr::unchecked(allowed_seller_address)],
            agreement_terms_hash: "mock-terms-hash".to_string(),
            token_denom: token_denom.into(),
            max_face_value_cents: Uint128::new(550000000),
            min_face_value_cents: Uint128::new(550000000),
            tick_size: Uint128::new(1000),
            dealers: vec![Addr::unchecked("dealer-address")],
            is_disabled: false,
        };
        save_contract_config(&mut deps.storage, &config).unwrap();

        save_buyer_state(
            &mut deps.storage,
            &Buyer {
                buyer_address: Addr::unchecked(buyer_address),
                has_accepted_pools: false,
            },
        )
        .unwrap();

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
        let buyer_address = "buyer-address";
        let token_denom = "test.forward.market.token";
        let info = mock_info(allowed_seller_address, &[]);
        let env = mock_env();
        let config = Config {
            is_private: true,
            allowed_sellers: vec![Addr::unchecked(allowed_seller_address)],
            agreement_terms_hash: "mock-terms-hash".to_string(),
            token_denom: token_denom.into(),
            max_face_value_cents: Uint128::new(550000000),
            min_face_value_cents: Uint128::new(550000000),
            tick_size: Uint128::new(1000),
            dealers: vec![Addr::unchecked("dealer-address")],
            is_disabled: false,
        };
        save_contract_config(&mut deps.storage, &config).unwrap();

        save_buyer_state(
            &mut deps.storage,
            &Buyer {
                buyer_address: Addr::unchecked(buyer_address),
                has_accepted_pools: false,
            },
        )
        .unwrap();

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
        let buyer_address = "buyer-address";
        let token_denom = "test.forward.market.token";
        let info = mock_info(buyer_address, &[]);
        let env = mock_env();
        let config = Config {
            is_private: true,
            allowed_sellers: vec![Addr::unchecked(allowed_seller_address)],
            agreement_terms_hash: "mock-terms-hash".to_string(),
            token_denom: token_denom.into(),
            max_face_value_cents: Uint128::new(550000000),
            min_face_value_cents: Uint128::new(350000000),
            tick_size: Uint128::new(1000),
            dealers: vec![Addr::unchecked("dealer-address")],
            is_disabled: false,
        };
        save_contract_config(&mut deps.storage, &config).unwrap();

        save_buyer_state(
            &mut deps.storage,
            &Buyer {
                buyer_address: Addr::unchecked(buyer_address),
                has_accepted_pools: false,
            },
        )
        .unwrap();

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

        match execute(deps.as_mut(), env.clone(), info, ContractDisable {}) {
            Ok(_) => {
                panic!(
                    "Failed to detect error when disabling the contract while there is a \
                finalized seller pool list"
                )
            }
            Err(error) => match error {
                ContractError::IllegalDisableRequest => {}
                _ => {
                    panic!(
                        "Unexpected error encountered when disabling a contract while there \
                        is a finalized seller pool list"
                    )
                }
            },
        }
    }

    #[test]
    fn disallow_all_executions_when_disabled() {
        let mut deps = mock_provenance_dependencies();
        let info = mock_info("", &[]);
        let env = mock_env();
        save_contract_config(
            &mut deps.storage,
            &Config {
                is_private: false,
                allowed_sellers: vec![],
                agreement_terms_hash: "mock-terms-hash".to_string(),
                token_denom: "denom".into(),
                max_face_value_cents: Uint128::new(550000000),
                min_face_value_cents: Uint128::new(550000000),
                tick_size: Uint128::new(1000),
                dealers: vec![Addr::unchecked("dealer-address")],
                is_disabled: true,
            },
        )
        .unwrap();
        save_buyer_state(
            &mut deps.storage,
            &Buyer {
                buyer_address: Addr::unchecked("buyer-address"),
                has_accepted_pools: false,
            },
        )
        .unwrap();
        [
            ContractDisable {},
            AddSeller {
                accepted_value_cents: Uint128::new(1),
                offer_hash: "mock-offer-hash".to_string(),
                agreement_terms_hash: "mock-terms-hash".to_string(),
            },
            RemoveAsSeller {},
            FinalizePools {
                pool_denoms: vec![],
            },
            DealerConfirm {},
            UpdateAgreementTermsHash {
                agreement_terms_hash: "".to_string(),
            },
            UpdateFaceValueCents {
                max_face_value_cents: Uint128::new(1),
                min_face_value_cents: Uint128::new(1),
                tick_size: Uint128::new(1),
            },
            UpdateAllowedSellers {
                allowed_sellers: vec![],
            },
            AcceptFinalizedPools {},
            RescindFinalizedPools {},
            DealerReset {},
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
