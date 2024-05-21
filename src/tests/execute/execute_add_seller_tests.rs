#[cfg(test)]
mod execute_add_seller_tests {
    use crate::contract::execute;
    use crate::error::ContractError;
    use crate::msg::ExecuteMsg::AddSeller;
    use crate::storage::state_store::{
        retrieve_seller_state, save_buyer_state, save_contract_config, save_seller_state, Buyer,
        Config, Seller,
    };
    use cosmwasm_std::testing::{mock_env, mock_info};
    use cosmwasm_std::{Addr, Attribute, CosmosMsg, Uint128};
    use provwasm_mocks::mock_provenance_dependencies;
    use provwasm_std::types::cosmos::base::v1beta1::Coin;
    use provwasm_std::types::provenance::marker::v1::{
        Access, AccessGrant, MarkerStatus, MarkerType, MsgActivateRequest, MsgAddMarkerRequest,
        MsgFinalizeRequest, MsgWithdrawRequest,
    };
    use std::ops::Div;

    #[test]
    fn add_seller_to_public_forward_market() {
        let mut deps = mock_provenance_dependencies();
        let info = mock_info("contract_seller", &[]);
        let env = mock_env();
        let buyer_address = "contract_buyer";
        let dealer_address = "dealer_address";
        let token_denom = "test.forward.market.token";
        let accepted_value_cents = Uint128::new(400000000);
        let add_seller_msg = AddSeller {
            accepted_value_cents,
            offer_hash: "mock-offer-hash".to_string(),
            agreement_terms_hash: "mock-terms-hash".to_string(),
        };
        let contract_address = env.contract.address.to_string();
        let tick_size = Uint128::new(1000);

        save_contract_config(
            &mut deps.storage,
            &Config {
                is_private: false,
                allowed_sellers: vec![],
                agreement_terms_hash: "mock-terms-hash".to_string(),
                token_denom: token_denom.into(),
                max_face_value_cents: Uint128::new(500000000),
                min_face_value_cents: Uint128::new(100000000),
                tick_size,
                dealers: vec![Addr::unchecked(dealer_address)],
                is_disabled: false,
            },
        )
        .unwrap();

        save_buyer_state(
            &mut deps.storage,
            &Buyer {
                buyer_address: Addr::unchecked("contract_buyer"),
                has_accepted_pools: false,
            },
        )
        .unwrap();

        match execute(deps.as_mut(), env, info, add_seller_msg) {
            Ok(response) => match retrieve_seller_state(&deps.storage) {
                Ok(seller_info) => {
                    let coin = Coin {
                        denom: token_denom.into(),
                        amount: accepted_value_cents.div(tick_size).to_string(),
                    };
                    assert_eq!(
                        response.messages[0].msg,
                        CosmosMsg::from(MsgAddMarkerRequest {
                            amount: Some(coin),
                            manager: contract_address.clone(),
                            from_address: contract_address.clone(),
                            status: MarkerStatus::Proposed as i32,
                            marker_type: MarkerType::Coin as i32,
                            access_list: vec![
                                AccessGrant {
                                    address: dealer_address.to_string(),
                                    permissions: vec![
                                        Access::Withdraw as i32,
                                        Access::Deposit as i32,
                                    ],
                                },
                                AccessGrant {
                                    address: contract_address.to_string(),
                                    permissions: vec![
                                        Access::Admin as i32,
                                        Access::Burn as i32,
                                        Access::Mint as i32,
                                        Access::Deposit as i32,
                                        Access::Withdraw as i32,
                                        Access::Delete as i32,
                                    ],
                                }
                            ],
                            supply_fixed: false,
                            allow_governance_control: true,
                            allow_forced_transfer: false,
                            required_attributes: vec![],
                        })
                    );
                    assert_eq!(
                        response.messages[1].msg,
                        CosmosMsg::from(MsgFinalizeRequest {
                            denom: token_denom.to_string(),
                            administrator: contract_address.clone(),
                        })
                    );
                    assert_eq!(
                        response.messages[2].msg,
                        CosmosMsg::from(MsgActivateRequest {
                            denom: token_denom.to_string(),
                            administrator: contract_address.clone(),
                        })
                    );

                    assert_eq!(
                        response.messages[3].msg,
                        CosmosMsg::from(MsgWithdrawRequest {
                            denom: token_denom.to_string(),
                            administrator: contract_address.clone(),
                            to_address: buyer_address.to_string(),
                            amount: vec![Coin {
                                denom: token_denom.to_string(),
                                amount: accepted_value_cents.div(tick_size).to_string(),
                            }],
                        })
                    );
                    let expected_seller_info = Seller {
                        seller_address: Addr::unchecked("contract_seller"),
                        accepted_value_cents,
                        pool_denoms: vec![],
                        offer_hash: "mock-offer-hash".to_string(),
                    };
                    assert_eq!(seller_info, expected_seller_info);
                    assert_eq!(
                        response.attributes[0],
                        Attribute::new("seller_state", format!("{:?}", expected_seller_info))
                    );
                }
                Err(_) => {
                    panic!("failed to retrieve seller data after adding the seller")
                }
            },
            Err(error) => {
                panic!("failed to add seller: {:?}", error)
            }
        }
    }

    #[test]
    fn add_seller_with_invalid_accepted_value() {
        let mut deps = mock_provenance_dependencies();
        let info = mock_info("contract_seller", &[]);
        let env = mock_env();
        let add_seller_msg = AddSeller {
            accepted_value_cents: Uint128::new(900000000),
            offer_hash: "mock-offer-hash".to_string(),
            agreement_terms_hash: "mock-terms-hash".to_string(),
        };

        save_contract_config(
            &mut deps.storage,
            &Config {
                is_private: false,
                allowed_sellers: vec![],
                agreement_terms_hash: "mock-terms-hash".to_string(),
                token_denom: "test.forward.market.token".into(),
                max_face_value_cents: Uint128::new(500000000),
                min_face_value_cents: Uint128::new(400000000),
                tick_size: Uint128::new(1000),
                dealers: vec![Addr::unchecked("dealer-address")],
                is_disabled: false,
            },
        )
        .unwrap();

        match execute(deps.as_mut(), env, info, add_seller_msg) {
            Ok(_) => {
                panic!("failure to return an error when using an accepted value greater than the face value")
            }
            Err(error) => match error {
                ContractError::AcceptedValueExceedsMaxFaceValue => {}
                _ => {
                    panic!("Unexpected error returned when using an accepted value greater than the face value")
                }
            },
        }
    }

    #[test]
    fn add_duplicate_seller() {
        let mut deps = mock_provenance_dependencies();
        let info = mock_info("contract_seller", &[]);
        let env = mock_env();
        save_contract_config(
            &mut deps.storage,
            &Config {
                is_private: false,
                allowed_sellers: vec![],
                agreement_terms_hash: "mock-terms-hash".to_string(),
                token_denom: "test.forward.market.token".into(),
                max_face_value_cents: Uint128::new(500000000),
                min_face_value_cents: Uint128::new(300000),
                tick_size: Uint128::new(1000),
                dealers: vec![Addr::unchecked("dealer-address")],
                is_disabled: false,
            },
        )
        .unwrap();

        save_seller_state(
            &mut deps.storage,
            &Seller {
                seller_address: Addr::unchecked("existing_seller"),
                accepted_value_cents: Uint128::new(100000000),
                pool_denoms: vec![],
                offer_hash: "mock-offer-hash".to_string(),
            },
        )
        .unwrap();

        let add_seller_msg = AddSeller {
            accepted_value_cents: Uint128::new(200000000),
            offer_hash: "mock-offer-hash".to_string(),
            agreement_terms_hash: "mock-terms-hash".to_string(),
        };
        match execute(deps.as_mut(), env, info, add_seller_msg) {
            Ok(_) => {
                panic!("failed to return error when a duplicate seller was added")
            }
            Err(error) => match error {
                ContractError::SellerAlreadyExists => {}
                _ => {
                    panic!("an unexpected error was returned when attempting to add a duplicate seller")
                }
            },
        }
    }

    #[test]
    fn add_seller_to_private_forward_market() {
        let mut deps = mock_provenance_dependencies();
        let info = mock_info("private-seller-0", &[]);
        let buyer_address = "contract_buyer";
        let env = mock_env();
        let accepted_value_cents = Uint128::new(100000000);
        let dealer_address = "dealer-address";
        let add_seller_msg = AddSeller {
            accepted_value_cents,
            offer_hash: "mock-offer-hash".to_string(),
            agreement_terms_hash: "mock-terms-hash".to_string(),
        };
        let token_denom = "test.forward.market.token";
        let contract_address = env.contract.address.to_string();
        let tick_size = Uint128::new(1000);

        save_contract_config(
            &mut deps.storage,
            &Config {
                is_private: true,
                allowed_sellers: vec![Addr::unchecked("private-seller-0")],
                agreement_terms_hash: "mock-terms-hash".to_string(),
                token_denom: token_denom.into(),
                max_face_value_cents: Uint128::new(1500000000),
                min_face_value_cents: Uint128::new(200000),
                tick_size,
                dealers: vec![Addr::unchecked(dealer_address)],
                is_disabled: false,
            },
        )
        .unwrap();

        save_buyer_state(
            &mut deps.storage,
            &Buyer {
                buyer_address: Addr::unchecked("contract_buyer"),
                has_accepted_pools: false,
            },
        )
        .unwrap();

        match execute(deps.as_mut(), env, info, add_seller_msg) {
            Ok(response) => {
                let coin = Coin {
                    denom: token_denom.into(),
                    amount: accepted_value_cents.div(tick_size).to_string(),
                };
                assert_eq!(
                    response.messages[0].msg,
                    CosmosMsg::from(MsgAddMarkerRequest {
                        amount: Some(coin),
                        manager: contract_address.clone(),
                        from_address: contract_address.clone(),
                        status: MarkerStatus::Proposed as i32,
                        marker_type: MarkerType::Coin as i32,
                        access_list: vec![
                            AccessGrant {
                                address: dealer_address.to_string(),
                                permissions: vec![Access::Withdraw as i32, Access::Deposit as i32,],
                            },
                            AccessGrant {
                                address: contract_address.to_string(),
                                permissions: vec![
                                    Access::Admin as i32,
                                    Access::Burn as i32,
                                    Access::Mint as i32,
                                    Access::Deposit as i32,
                                    Access::Withdraw as i32,
                                    Access::Delete as i32,
                                ],
                            }
                        ],
                        supply_fixed: false,
                        allow_governance_control: true,
                        allow_forced_transfer: false,
                        required_attributes: vec![],
                    })
                );
                assert_eq!(
                    response.messages[1].msg,
                    CosmosMsg::from(MsgFinalizeRequest {
                        denom: token_denom.to_string(),
                        administrator: contract_address.clone(),
                    })
                );
                assert_eq!(
                    response.messages[2].msg,
                    CosmosMsg::from(MsgActivateRequest {
                        denom: token_denom.to_string(),
                        administrator: contract_address.clone(),
                    })
                );

                assert_eq!(
                    response.messages[3].msg,
                    CosmosMsg::from(MsgWithdrawRequest {
                        denom: token_denom.to_string(),
                        administrator: contract_address.clone(),
                        to_address: buyer_address.to_string(),
                        amount: vec![Coin {
                            denom: token_denom.to_string(),
                            amount: accepted_value_cents.div(tick_size).to_string(),
                        }],
                    })
                );
            }
            Err(error) => {
                panic!("failed to add seller: {:?}", error)
            }
        }
    }

    #[test]
    fn add_invalid_seller_to_private_forward_market() {
        let mut deps = mock_provenance_dependencies();
        let info = mock_info("private-seller-0", &[]);
        let env = mock_env();
        let add_seller_msg = AddSeller {
            accepted_value_cents: Uint128::new(100000000),
            offer_hash: "mock-offer-hash".to_string(),
            agreement_terms_hash: "mock-terms-hash".to_string(),
        };

        save_contract_config(
            &mut deps.storage,
            &Config {
                is_private: true,
                allowed_sellers: vec![],
                agreement_terms_hash: "mock-terms-hash".to_string(),
                token_denom: "test.forward.market.token".into(),
                max_face_value_cents: Uint128::new(500000000),
                min_face_value_cents: Uint128::new(300000000),
                tick_size: Uint128::new(1000),
                dealers: vec![Addr::unchecked("dealer-address")],
                is_disabled: false,
            },
        )
        .unwrap();

        match execute(deps.as_mut(), env, info, add_seller_msg) {
            Ok(_) => {
                panic!("failed to return an error when adding a seller that is not in the allowed list of sellers for \
                    a private contract")
            }
            Err(error) => match error {
                ContractError::UnauthorizedPrivateSeller => {}
                _ => {
                    panic!("an unexpected error was returned when attempting to add an invalid seller to a private \
                        contract")
                }
            },
        }
    }

    #[test]
    fn add_seller_with_invalid_agreement_hash() {
        let mut deps = mock_provenance_dependencies();
        let info = mock_info("private-seller-0", &[]);
        let env = mock_env();
        let add_seller_msg = AddSeller {
            accepted_value_cents: Uint128::new(400000000),
            offer_hash: "mock-offer-hash".to_string(),
            agreement_terms_hash: "hash-A".to_string(),
        };

        save_contract_config(
            &mut deps.storage,
            &Config {
                is_private: true,
                allowed_sellers: vec![Addr::unchecked("private-seller-0")],
                agreement_terms_hash: "hash-B".to_string(),
                token_denom: "test.forward.market.token".into(),
                max_face_value_cents: Uint128::new(500000000),
                min_face_value_cents: Uint128::new(300000000),
                tick_size: Uint128::new(1000),
                dealers: vec![Addr::unchecked("dealer-address")],
                is_disabled: false,
            },
        )
        .unwrap();

        match execute(deps.as_mut(), env, info, add_seller_msg) {
            Ok(_) => {
                panic!("failed to return an error when adding a seller with a stale contract hash")
            }
            Err(error) => {
                match error {
                    ContractError::InvalidAgreementTermsHash => {}
                    _ => {
                        panic!("an unexpected error was returned when attempting to add a seller with \
                    a stale contract hash")
                    }
                }
            }
        }
    }
}
