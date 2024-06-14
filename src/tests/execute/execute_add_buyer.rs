#[cfg(test)]
mod execute_add_bidder_tests {
    use cosmwasm_std::testing::{mock_env, mock_info};
    use cosmwasm_std::{Addr, Response, Uint128};
    use provwasm_mocks::mock_provenance_dependencies;
    use crate::contract::execute;
    use crate::error::ContractError;
    use crate::msg::ExecuteMsg::AddBuyer;
    use crate::query::contract_state::query_contract_state;
    use crate::storage::state_store::{Buyer, BuyerList, Config, save_buyer_state, save_contract_config, Seller};
    use crate::version_info::{set_version_info, VersionInfoV1};

    #[test]
    fn add_bidders_to_public_forward_market() {
        let mut deps = mock_provenance_dependencies();
        let info = mock_info("bidder_address", &[]);
        let env = mock_env();

        save_contract_config(
            &mut deps.storage,
            &Config {
                use_private_sellers: false,
                use_private_buyers: false,
                allowed_sellers: vec![],
                allowed_buyers: vec![],
                token_denom: "test.forward.market.token".to_string(),
                max_face_value_cents: Uint128::new(500000000),
                min_face_value_cents: Uint128::new(100000000),
                tick_size: Uint128::new(1000),
                dealers: vec![Addr::unchecked("dealer-address")],
                is_disabled: false,
                max_buyer_count: 3,
                contract_admin: Addr::unchecked("contract-admin")
            },
        ).unwrap();

        set_version_info(
            &mut deps.storage,
            &VersionInfoV1 {
                definition: "mock".to_string(),
                version: "0.0.0".to_string(),
            },
        ).unwrap();

        let existing_bidder = Buyer {
            buyer_address: Addr::unchecked("existing-buyer-address"),
            agreement_terms_hash: "mock-hash-existing-buyers".to_string(),
        };
        save_buyer_state(&mut deps.storage, &BuyerList {
            buyers: vec![
                existing_bidder.clone()
            ],
        }).unwrap();

        let add_bidder_message = AddBuyer {
            agreement_terms_hash: "buyer-mock-hash".to_string(),
        };
        match execute(deps.as_mut(), env, info, add_bidder_message) {
            Ok(_) => {
                assert_eq!(
                    query_contract_state(deps.as_ref()).unwrap().buyers,
                    vec![
                        existing_bidder.clone(),
                        Buyer {
                            buyer_address: Addr::unchecked("bidder_address"),
                            agreement_terms_hash: "buyer-mock-hash".to_string(),
                        }
                    ]
                );
            }
            Err(error) => {
                panic!("failed to add bidder: {:?}", error)
            }
        }
    }

    #[test]
    fn add_bidders_to_private_forward_market() {
        let mut deps = mock_provenance_dependencies();
        let info = mock_info("bidder_address", &[]);
        let env = mock_env();

        save_contract_config(
            &mut deps.storage,
            &Config {
                use_private_sellers: false,
                use_private_buyers: true,
                allowed_sellers: vec![],
                allowed_buyers: vec![Addr::unchecked("bidder_address")],
                token_denom: "test.forward.market.token".to_string(),
                max_face_value_cents: Uint128::new(500000000),
                min_face_value_cents: Uint128::new(100000000),
                tick_size: Uint128::new(1000),
                dealers: vec![Addr::unchecked("dealer-address")],
                is_disabled: false,
                max_buyer_count: 3,
                contract_admin: Addr::unchecked("contract-admin")
            },
        ).unwrap();

        set_version_info(
            &mut deps.storage,
            &VersionInfoV1 {
                definition: "mock".to_string(),
                version: "0.0.0".to_string(),
            },
        ).unwrap();

        save_buyer_state(&mut deps.storage, &BuyerList {
            buyers: vec![],
        }).unwrap();

        let add_bidder_message = AddBuyer {
            agreement_terms_hash: "buyer-mock-hash".to_string(),
        };
        match execute(deps.as_mut(), env, info, add_bidder_message) {
            Ok(_) => {
                assert_eq!(
                    query_contract_state(deps.as_ref()).unwrap().buyers,
                    vec![
                        Buyer {
                            buyer_address: Addr::unchecked("bidder_address"),
                            agreement_terms_hash: "buyer-mock-hash".to_string(),
                        }
                    ]
                );
            }
            Err(error) => {
                panic!("failed to add bidder: {:?}", error)
            }
        }
    }

    #[test]
    fn reject_disallowed_bidder_private_forward_market() {
        let mut deps = mock_provenance_dependencies();
        let info = mock_info("bidder_address", &[]);
        let env = mock_env();

        save_contract_config(
            &mut deps.storage,
            &Config {
                use_private_sellers: false,
                use_private_buyers: true,
                allowed_sellers: vec![],
                allowed_buyers: vec![Addr::unchecked("bidder_address_0")],
                token_denom: "test.forward.market.token".to_string(),
                max_face_value_cents: Uint128::new(500000000),
                min_face_value_cents: Uint128::new(100000000),
                tick_size: Uint128::new(1000),
                dealers: vec![Addr::unchecked("dealer-address")],
                is_disabled: false,
                max_buyer_count: 3,
                contract_admin: Addr::unchecked("contract-admin")
            },
        ).unwrap();

        set_version_info(
            &mut deps.storage,
            &VersionInfoV1 {
                definition: "mock".to_string(),
                version: "0.0.0".to_string(),
            },
        ).unwrap();

        save_buyer_state(&mut deps.storage, &BuyerList {
            buyers: vec![],
        }).unwrap();

        let add_bidder_message = AddBuyer {
            agreement_terms_hash: "buyer-mock-hash".to_string(),
        };
        match execute(deps.as_mut(), env, info, add_bidder_message) {
            Ok(_) => {
                panic!("Failed to detect error when an address not in the allowed buyer list submitted a bid")
            }
            Err(error) => {
                match error {
                    ContractError::UnauthorizedPrivateBuyer => {}
                    _ => {
                        panic!("Unexpected error returned when attempting to add an unauthorized bidder")
                    }
                }
            }
        }
    }

    #[test]
    fn reject_over_max_bidders_private_forward_market() {
        let mut deps = mock_provenance_dependencies();
        let info = mock_info("bidder_address", &[]);
        let env = mock_env();

        save_contract_config(
            &mut deps.storage,
            &Config {
                use_private_sellers: false,
                use_private_buyers: true,
                allowed_sellers: vec![],
                allowed_buyers: vec![Addr::unchecked("bidder_address")],
                token_denom: "test.forward.market.token".to_string(),
                max_face_value_cents: Uint128::new(500000000),
                min_face_value_cents: Uint128::new(100000000),
                tick_size: Uint128::new(1000),
                dealers: vec![Addr::unchecked("dealer-address")],
                is_disabled: false,
                max_buyer_count: 2,
                contract_admin: Addr::unchecked("contract-admin")
            },
        ).unwrap();

        set_version_info(
            &mut deps.storage,
            &VersionInfoV1 {
                definition: "mock".to_string(),
                version: "0.0.0".to_string(),
            },
        ).unwrap();

        save_buyer_state(&mut deps.storage, &BuyerList {
            buyers: vec![
                Buyer {
                    buyer_address: Addr::unchecked("existing-buyer-address-0"),
                    agreement_terms_hash: "mock-hash-existing-buyers-0".to_string(),
                },
                Buyer {
                    buyer_address: Addr::unchecked("existing-buyer-address-1"),
                    agreement_terms_hash: "mock-hash-existing-buyers-1".to_string(),
                }
            ],
        }).unwrap();

        let add_bidder_message = AddBuyer {
            agreement_terms_hash: "buyer-mock-hash".to_string(),
        };
        match execute(deps.as_mut(), env, info, add_bidder_message) {
            Ok(_) => {
                panic!("Failed to detect error when max bidder threshold has breached")
            }
            Err(error) => {
                match error {
                    ContractError::MaxPrivateBuyersReached => {}
                    _ => {
                        panic!("Unexpected error returned when attempting to breach max bidder threshold")
                    }
                }
            }
        }
    }
}

