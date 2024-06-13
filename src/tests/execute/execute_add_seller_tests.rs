#[cfg(test)]
mod execute_add_seller_tests {
    use crate::contract::execute;
    use crate::error::ContractError;
    use crate::msg::ExecuteMsg::AddSeller;
    use crate::storage::state_store::{save_buyer_state, save_contract_config, save_seller_state, Buyer, Config, Seller, BuyerList};
    use cosmwasm_std::testing::{mock_env, mock_info};
    use cosmwasm_std::{Addr, Uint128};
    use provwasm_mocks::mock_provenance_dependencies;
    use crate::query::contract_state::query_contract_state;
    use crate::version_info::{set_version_info, VersionInfoV1};

    #[test]
    fn add_seller_to_public_forward_market() {
        let mut deps = mock_provenance_dependencies();
        let info = mock_info("private-seller-0", &[]);
        let env = mock_env();
        let dealer_address = "dealer_address";
        let buyer_address = "buyer_address";
        let token_denom = "test.forward.market.token";
        let accepted_value_cents = Uint128::new(400000000);
        let add_seller_msg = AddSeller {
            accepted_value_cents,
            offer_hash: "mock-offer-hash".to_string(),
        };
        let contract_address = env.contract.address.to_string();
        let tick_size = Uint128::new(1000);

        save_contract_config(
            &mut deps.storage,
            &Config {
                use_private_sellers: false,
                use_private_buyers: false,
                allowed_sellers: vec![],
                allowed_buyers: vec![],
                token_denom: token_denom.into(),
                max_face_value_cents: Uint128::new(500000000),
                min_face_value_cents: Uint128::new(100000000),
                tick_size,
                dealers: vec![Addr::unchecked(dealer_address)],
                is_disabled: false,
                max_buyer_count: 3,
                contract_admin: Addr::unchecked("contract-admin")
            },
        )
        .unwrap();

        set_version_info(
            &mut deps.storage,
            &VersionInfoV1 {
                definition: "mock".to_string(),
                version: "0.0.0".to_string(),
            },
        ).unwrap();

        save_buyer_state(&mut deps.storage, &BuyerList {
            buyers: vec![Buyer {
                buyer_address: Addr::unchecked(buyer_address),
                agreement_terms_hash: "".to_string(),
            }],
        }).unwrap();

        match execute(deps.as_mut(), env, info, add_seller_msg) {
            Ok(_) => {
                assert_eq!(
                    query_contract_state(deps.as_ref()).unwrap().seller.unwrap(),
                    Seller {
                        seller_address: Addr::unchecked("private-seller-0"),
                        accepted_value_cents,
                        pool_denoms: vec![],
                        offer_hash: "mock-offer-hash".to_string(),
                    }
                );
            }
            Err(error) => {
                panic!("failed to add seller: {:?}", error)
            }
        }
    }

    #[test]
    fn add_seller_with_invalid_accepted_value() {
        let mut deps = mock_provenance_dependencies();
        let contract_admin = "contract_admin";
        let info = mock_info(contract_admin, &[]);
        let env = mock_env();
        let add_seller_msg = AddSeller {
            accepted_value_cents: Uint128::new(900000000),
            offer_hash: "mock-offer-hash".to_string()
        };

        save_contract_config(
            &mut deps.storage,
            &Config {
                use_private_sellers: false,
                use_private_buyers: false,
                allowed_sellers: vec![],
                allowed_buyers: vec![],
                token_denom: "test.forward.market.token".into(),
                max_face_value_cents: Uint128::new(500000000),
                min_face_value_cents: Uint128::new(400000000),
                tick_size: Uint128::new(1000),
                dealers: vec![Addr::unchecked("dealer-address")],
                is_disabled: false,
                max_buyer_count: 10,
                contract_admin: Addr::unchecked(contract_admin)
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
        let contract_admin = "contract_admin";
        let info = mock_info(contract_admin, &[]);
        let env = mock_env();
        save_contract_config(
            &mut deps.storage,
            &Config {
                use_private_sellers: false,
                use_private_buyers: false,
                allowed_sellers: vec![],
                allowed_buyers: vec![],
                token_denom: "test.forward.market.token".into(),
                max_face_value_cents: Uint128::new(500000000),
                min_face_value_cents: Uint128::new(300000),
                tick_size: Uint128::new(1000),
                dealers: vec![Addr::unchecked("dealer-address")],
                is_disabled: false,
                max_buyer_count: 2,
                contract_admin: Addr::unchecked(contract_admin)
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
        let contract_admin = "contract_admin";
        let info = mock_info("private-seller-0", &[]);
        let buyer_address = "contract_buyer";
        let env = mock_env();
        let accepted_value_cents = Uint128::new(100000000);
        let dealer_address = "dealer-address";
        let add_seller_msg = AddSeller {
            accepted_value_cents,
            offer_hash: "mock-offer-hash".to_string(),
        };
        let token_denom = "test.forward.market.token";
        let contract_address = env.contract.address.to_string();
        let tick_size = Uint128::new(1000);

        save_contract_config(
            &mut deps.storage,
            &Config {
                use_private_sellers: true,
                use_private_buyers: true,
                allowed_sellers: vec![Addr::unchecked("private-seller-0")],
                allowed_buyers: vec![],
                token_denom: token_denom.into(),
                max_face_value_cents: Uint128::new(1500000000),
                min_face_value_cents: Uint128::new(200000),
                tick_size,
                dealers: vec![Addr::unchecked(dealer_address)],
                is_disabled: false,
                max_buyer_count: 2,
                contract_admin: Addr::unchecked(contract_admin)
            },
        )
        .unwrap();

        set_version_info(
            &mut deps.storage,
            &VersionInfoV1 {
                definition: "mock".to_string(),
                version: "0.0.0".to_string(),
            },
        ).unwrap();

        save_buyer_state(&mut deps.storage, &BuyerList {
            buyers: vec![Buyer {
                buyer_address: Addr::unchecked(buyer_address),
                agreement_terms_hash: "".to_string(),
            }],
        }).unwrap();

        match execute(deps.as_mut(), env, info, add_seller_msg) {
            Ok(_) => {
                assert_eq!(
                    query_contract_state(deps.as_ref()).unwrap().seller.unwrap(),
                    Seller {
                        seller_address: Addr::unchecked("private-seller-0"),
                        accepted_value_cents,
                        pool_denoms: vec![],
                        offer_hash: "mock-offer-hash".to_string(),
                    }
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
        let contract_admin = "contract-admin";
        let info = mock_info(contract_admin, &[]);
        let env = mock_env();
        let add_seller_msg = AddSeller {
            accepted_value_cents: Uint128::new(100000000),
            offer_hash: "mock-offer-hash".to_string(),
        };

        save_contract_config(
            &mut deps.storage,
            &Config {
                use_private_sellers: true,
                use_private_buyers: false,
                allowed_sellers: vec![],
                allowed_buyers: vec![],
                token_denom: "test.forward.market.token".into(),
                max_face_value_cents: Uint128::new(500000000),
                min_face_value_cents: Uint128::new(300000000),
                tick_size: Uint128::new(1000),
                dealers: vec![Addr::unchecked("dealer-address")],
                is_disabled: false,
                max_buyer_count: 5,
                contract_admin: Addr::unchecked(contract_admin)
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

    // TODO: Move this and update it to test accepting a buyer bid with the wrong hash
    // #[test]
    // fn add_seller_with_invalid_agreement_hash() {
    //     let mut deps = mock_provenance_dependencies();
    //     let contract_admin = "contract-admin";
    //     let info = mock_info(contract_admin, &[]);
    //     let env = mock_env();
    //     let add_seller_msg = AddSeller {
    //         accepted_value_cents: Uint128::new(400000000),
    //         offer_hash: "mock-offer-hash".to_string(),
    //     };
    //
    //     save_contract_config(
    //         &mut deps.storage,
    //         &Config {
    //             use_private_sellers: true,
    //             use_private_buyers: false,
    //             allowed_sellers: vec![Addr::unchecked("private-seller-0")],
    //             allowed_buyers: vec![],
    //             token_denom: "test.forward.market.token".into(),
    //             max_face_value_cents: Uint128::new(500000000),
    //             min_face_value_cents: Uint128::new(300000000),
    //             tick_size: Uint128::new(1000),
    //             dealers: vec![Addr::unchecked("dealer-address")],
    //             is_disabled: false,
    //             max_buyer_count: 5,
    //             contract_admin: Addr::unchecked(contract_admin)
    //         },
    //     )
    //     .unwrap();
    //
    //     match execute(deps.as_mut(), env, info, add_seller_msg) {
    //         Ok(_) => {
    //             panic!("failed to return an error when adding a seller with a stale contract hash")
    //         }
    //         Err(error) => {
    //             match error {
    //                 ContractError::InvalidAgreementTermsHash => {}
    //                 _ => {
    //                     panic!("an unexpected error was returned when attempting to add a seller with \
    //                 a stale contract hash")
    //                 }
    //             }
    //         }
    //     }
    // }
}
