#[cfg(test)]
mod execute_dealer_confirm_tests {
    use crate::contract::execute;
    use crate::error::ContractError;
    use crate::storage::state_store::{
        retrieve_optional_settlement_data_state, save_buyer_state, save_contract_config,
        save_seller_state, save_settlement_data_state, Buyer, Config, Seller, SettlementData,
    };
    use cosmwasm_std::testing::{mock_env, mock_info};
    use cosmwasm_std::{to_json_binary, Binary, ContractResult, SystemResult};
    use cosmwasm_std::{Addr, CosmosMsg, Uint128};
    use prost::Message;
    use provwasm_mocks::mock_provenance_dependencies;
    use provwasm_std::shim::Any;
    use provwasm_std::types::cosmos::auth::v1beta1::BaseAccount;
    use provwasm_std::types::cosmos::bank::v1beta1::MsgSend;
    use provwasm_std::types::cosmos::base::v1beta1::Coin;

    use provwasm_std::types::provenance::marker::v1::{
        AccessGrant, MarkerAccount, MarkerStatus, MarkerType, QueryMarkerRequest,
        QueryMarkerResponse,
    };
    use provwasm_std::types::provenance::metadata::v1::{
        MsgUpdateValueOwnersRequest, ValueOwnershipRequest, ValueOwnershipResponse,
    };
    use cosmwasm_std::Coin as CosmwasmCoin;

    use crate::msg::ExecuteMsg::{
        AcceptFinalizedPools, AddSeller, ContractDisable, DealerConfirm, DealerReset,
        FinalizePools, RemoveAsSeller, RescindFinalizedPools, UpdateAgreementTermsHash,
        UpdateAllowedSellers, UpdateFaceValueCents,
    };
    use uuid::Uuid;

    #[test]
    fn execute_dealer_confirm() {
        let mut deps = mock_provenance_dependencies();
        let dealer_address = "dealer-address";
        let seller_address = "allowed-seller-0";
        let buyer_address = "contract_buyer";
        let token_denom = "test.forward.market.token";
        let scope_id = "8e6caea3-c91f-4d59-9741-fe6b665b2f14";
        let info = mock_info(dealer_address, &[]);
        let env = mock_env();
        save_contract_config(
            &mut deps.storage,
            &Config {
                is_private: true,
                allowed_sellers: vec![Addr::unchecked(seller_address)],
                agreement_terms_hash: "mock-terms-hash".to_string(),
                token_denom: token_denom.into(),
                max_face_value_cents: Uint128::new(650000000),
                min_face_value_cents: Uint128::new(100000),
                tick_size: Uint128::new(1000),
                dealers: vec![Addr::unchecked(dealer_address)],
                is_disabled: false,
            },
        )
        .unwrap();

        save_buyer_state(
            &mut deps.storage,
            &Buyer {
                buyer_address: Addr::unchecked(buyer_address),
                has_accepted_pools: true,
            },
        )
        .unwrap();

        save_seller_state(
            &mut deps.storage,
            &Seller {
                seller_address: Addr::unchecked(seller_address),
                accepted_value_cents: Uint128::new(550000000),
                pool_coins: vec![
                    CosmwasmCoin {
                        denom: "test.token.asset.pool.0".to_string(),
                        amount: Uint128::new(1),
                    }
                ],
                offer_hash: "mock-offer-hash".to_string(),
            },
        )
        .unwrap();

        let cb = Box::new(|bin: &Binary| -> SystemResult<ContractResult<Binary>> {
            let message = QueryMarkerRequest::try_from(bin.clone()).unwrap();

            let expected_marker = MarkerAccount {
                base_account: Some(BaseAccount {
                    address: "base_addr".to_string(),
                    pub_key: None,
                    account_number: 1,
                    sequence: 0,
                }),
                manager: "".to_string(),
                access_control: vec![AccessGrant {
                    address: "".to_string(),
                    permissions: vec![1, 2, 3, 4, 5, 6, 7],
                }],
                status: MarkerStatus::Active.into(),
                denom: message.id.to_string(),
                supply: "1".to_string(),
                marker_type: MarkerType::Coin.into(),
                supply_fixed: false,
                allow_governance_control: false,
                allow_forced_transfer: false,
                required_attributes: vec![],
            };

            let response = QueryMarkerResponse {
                marker: Some(Any {
                    type_url: "/provenance.marker.v1.MarkerAccount".to_string(),
                    value: expected_marker.encode_to_vec(),
                }),
            };

            let binary = to_json_binary(&response).unwrap();
            SystemResult::Ok(ContractResult::Ok(binary))
        });

        deps.querier
            .registered_custom_queries
            .insert("/provenance.marker.v1.Query/Marker".to_string(), cb);

        match execute(
            deps.as_mut(),
            env.clone(),
            info,
            crate::msg::ExecuteMsg::DealerConfirm {},
        ) {
            Ok(response) => {
                assert_eq!(
                    response.messages[0].msg,
                    CosmosMsg::from(MsgSend {

                        // scope_ids: vec![scope(Uuid::parse_str(scope_id).unwrap()).unwrap().bytes],
                        // value_owner_address: "base_addr".to_string(),
                        // signers: vec![env.clone().contract.address.to_string()]
                        from_address: env.clone().contract.address.to_string(),
                        to_address: "base_addr".to_string(),
                        amount: vec![Coin {
                            denom: "test.token.asset.pool.0".to_string(),
                            amount: "1".to_string(),
                        }],
                    })
                );

                let expected_settlement_data = SettlementData {
                    block_height: 12345,
                    settling_dealer: Addr::unchecked(dealer_address),
                };
                assert_eq!(
                    expected_settlement_data,
                    retrieve_optional_settlement_data_state(&deps.storage)
                        .unwrap()
                        .unwrap()
                )
            }
            Err(error) => {
                panic!(
                    "failed to confirm the transaction as the seller: {:?}",
                    error
                )
            }
        }
    }

    #[test]
    fn execute_seller_confirm_invalid_seller_state() {
        let mut deps = mock_provenance_dependencies();
        let dealer_address = "dealer-address";
        let seller_address = "allowed-seller-0";
        let buyer_address = "contract_buyer";
        let token_denom = "test.forward.market.token";
        let info = mock_info(dealer_address, &[]);
        let env = mock_env();
        save_contract_config(
            &mut deps.storage,
            &Config {
                is_private: true,
                allowed_sellers: vec![Addr::unchecked("different_seller")],
                agreement_terms_hash: "mock-terms-hash".to_string(),
                token_denom: token_denom.into(),
                max_face_value_cents: Uint128::new(650000000),
                min_face_value_cents: Uint128::new(100000),
                tick_size: Uint128::new(1000),
                dealers: vec![Addr::unchecked(dealer_address)],
                is_disabled: false,
            },
        )
        .unwrap();

        save_buyer_state(
            &mut deps.storage,
            &Buyer {
                buyer_address: Addr::unchecked(buyer_address),
                has_accepted_pools: true,
            },
        )
        .unwrap();

        save_seller_state(
            &mut deps.storage,
            &Seller {
                seller_address: Addr::unchecked(seller_address),
                accepted_value_cents: Uint128::new(550000000),
                pool_coins: vec![
                    CosmwasmCoin{
                        denom: "test.token.asset.pool.0".to_string(),
                        amount: Uint128::new(1),
                    }
                ],
                offer_hash: "mock-offer-hash".to_string(),
            },
        )
        .unwrap();

        match execute(
            deps.as_mut(),
            env.clone(),
            info,
            crate::msg::ExecuteMsg::DealerConfirm {},
        ) {
            Ok(_) => {
                panic!(
                    "failed to return an error when an invalid seller state (seller is not \
                    included in allowed sellers) was encountered"
                )
            }
            Err(error) => match error {
                ContractError::UnauthorizedPrivateSeller => {}
                _ => {
                    panic!(
                        "an unexpected error was returned when an invalid seller state (seller \
                            is not included in allowed sellers) was encountered"
                    )
                }
            },
        }
    }

    #[test]
    fn execute_seller_confirm_unauthorized_seller() {
        let mut deps = mock_provenance_dependencies();
        let dealer_address = "dealer-address";
        let seller_address = "allowed-seller-0";
        let buyer_address = "contract_buyer";
        let token_denom = "test.forward.market.token";
        let info = mock_info("not-the-dealer", &[]);
        let env = mock_env();
        save_contract_config(
            &mut deps.storage,
            &Config {
                is_private: true,
                allowed_sellers: vec![Addr::unchecked("different_seller")],
                agreement_terms_hash: "mock-terms-hash".to_string(),
                token_denom: token_denom.into(),
                max_face_value_cents: Uint128::new(650000000),
                min_face_value_cents: Uint128::new(100000),
                tick_size: Uint128::new(1000),
                dealers: vec![Addr::unchecked(dealer_address)],
                is_disabled: false,
            },
        )
        .unwrap();

        save_buyer_state(
            &mut deps.storage,
            &Buyer {
                buyer_address: Addr::unchecked(buyer_address),
                has_accepted_pools: true,
            },
        )
        .unwrap();

        save_seller_state(
            &mut deps.storage,
            &Seller {
                seller_address: Addr::unchecked(seller_address),
                accepted_value_cents: Uint128::new(550000000),
                pool_coins: vec![
                    CosmwasmCoin {
                        denom: "test.token.asset.pool.0".to_string(),
                        amount: Uint128::new(1),
                    }
                ],
                offer_hash: "mock-offer-hash".to_string(),
            },
        )
        .unwrap();
        match execute(
            deps.as_mut(),
            env.clone(),
            info,
            crate::msg::ExecuteMsg::DealerConfirm {},
        ) {
            Ok(_) => {
                panic!(
                    "failed to return an error when an unauthorized seller attempted to confirm the contract"
                )
            }
            Err(error) => match error {
                ContractError::IllegalConfirmationRequest => {}
                _ => {
                    panic!(
                            "an unexpected error was returned when an unauthorized seller attempted to confirm the \
                            contract")
                }
            },
        }
    }

    #[test]
    fn disallow_all_executions_after_settlement() {
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
                is_disabled: false,
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
        save_settlement_data_state(
            &mut deps.storage,
            &SettlementData {
                block_height: 1,
                settling_dealer: Addr::unchecked("dealer-address"),
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
            FinalizePools {},
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
                    ContractError::IllegalContractExecution => {}
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
