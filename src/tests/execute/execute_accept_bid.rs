#[cfg(test)]
mod execute_accept_buyer_tests {
    use cosmwasm_std::{Addr, Uint128};
    use cosmwasm_std::testing::{mock_env, mock_info};
    use provwasm_mocks::mock_provenance_dependencies;
    use crate::contract::execute;
    use crate::error::ContractError;
    use crate::msg::ExecuteMsg::AcceptBid;
    use crate::query::contract_state::query_contract_state;
    use crate::storage::state_store::{Bid, BidList, Config, save_bid_list_state, save_contract_config, save_seller_state, save_transaction_state, Seller, TransactionState};
    use crate::version_info::{set_version_info, VersionInfoV1};

    #[test]
    fn accept_bid() {
        let mut deps = mock_provenance_dependencies();
        let seller_address = Addr::unchecked("seller_address");
        let info = mock_info("seller_address", &[]);
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
                max_bid_count: 2,
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

        save_seller_state(&mut deps.storage, &Seller {
            seller_address,
            accepted_value_cents: Uint128::new(100000000),
            pool_denoms: vec!["mock.denom".to_string()],
            offer_hash: "mock-offer-hash".to_string(),
        }).unwrap();

        save_bid_list_state(&mut deps.storage, &BidList {
            bids: vec![
                Bid {
                    buyer_address: Addr::unchecked("existing-buyer-address-0"),
                    agreement_terms_hash: "mock-hash-existing-buyers-0".to_string(),
                },
                Bid {
                    buyer_address: Addr::unchecked("existing-buyer-address-1"),
                    agreement_terms_hash: "mock-hash-existing-buyers-1".to_string(),
                }
            ],
        }).unwrap();

        let accept_bid_message = AcceptBid {
            bidder_address: "existing-buyer-address-0".to_string(),
            agreement_terms_hash: "mock-hash-existing-buyers-0".to_string(),
        };
        match execute(deps.as_mut(), env, info, accept_bid_message) {
            Ok(_) => {
                assert_eq!(
                    query_contract_state(deps.as_ref()).unwrap().transaction_state.unwrap(),
                    TransactionState {
                        buyer_address: Addr::unchecked("existing-buyer-address-0"),
                        buyer_has_accepted_pools: false,
                        agreement_terms_hash: "mock-hash-existing-buyers-0".to_string(),
                    }
                );
            }
            Err(error) => {
                panic!("failed to add bidder: {:?}", error)
            }
        }
    }

    #[test]
    fn accept_nonexistent_buyer() {
        let mut deps = mock_provenance_dependencies();
        let seller_address = Addr::unchecked("seller_address");
        let info = mock_info("seller_address", &[]);
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
                max_bid_count: 2,
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

        save_seller_state(&mut deps.storage, &Seller {
            seller_address,
            accepted_value_cents: Uint128::new(100000000),
            pool_denoms: vec!["mock.denom".to_string()],
            offer_hash: "mock-offer-hash".to_string(),
        }).unwrap();

        save_bid_list_state(&mut deps.storage, &BidList {
            bids: vec![
                Bid {
                    buyer_address: Addr::unchecked("existing-buyer-address-0"),
                    agreement_terms_hash: "mock-hash-existing-buyers-0".to_string(),
                }
            ],
        }).unwrap();

        let accept_buyer_message = AcceptBid {
            bidder_address: "non-existing-buyer-address".to_string(),
            agreement_terms_hash: "mock-hash".to_string(),
        };
        match execute(deps.as_mut(), env, info, accept_buyer_message) {
            Ok(_) => {
                panic!("failed to detect error when accepting a buyer with an address not in the bidder list")
            }
            Err(error) => {
                match error {
                    ContractError::BidDoesNotExist { address } => {
                        assert_eq!(address, "non-existing-buyer-address".to_string())
                    }
                    _ => {
                        panic!("unexpected error encountered when accepting a buyer with an address not in the bidder list")
                    }
                }
            }
        }
    }

    #[test]
    fn accept_buyer_with_invalid_hash() {
        let mut deps = mock_provenance_dependencies();
        let seller_address = Addr::unchecked("seller_address");
        let info = mock_info("seller_address", &[]);
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
                max_bid_count: 2,
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

        save_seller_state(&mut deps.storage, &Seller {
            seller_address,
            accepted_value_cents: Uint128::new(100000000),
            pool_denoms: vec!["mock.denom".to_string()],
            offer_hash: "mock-offer-hash".to_string(),
        }).unwrap();

        save_bid_list_state(&mut deps.storage, &BidList {
            bids: vec![
                Bid {
                    buyer_address: Addr::unchecked("existing-buyer-address-0"),
                    agreement_terms_hash: "mock-hash-existing-buyers-0".to_string(),
                }
            ],
        }).unwrap();

        let accept_buyer_message = AcceptBid {
            bidder_address: "existing-buyer-address-0".to_string(),
            agreement_terms_hash: "stale-hash".to_string(),
        };
        match execute(deps.as_mut(), env, info, accept_buyer_message) {
            Ok(_) => {
                panic!("failed to detect error when accepting a buyer with an invalid agreement terms hash")
            }
            Err(error) => {
                match error {
                    ContractError::InvalidAgreementTermsHash => {}
                    _ => {
                        panic!("unexpected error encountered when accepting a buyer with an invalid agreement terms hash")
                    }
                }
            }
        }
    }

    #[test]
    fn accept_buyer_when_buyer_previously_accepted() {
        let mut deps = mock_provenance_dependencies();
        let seller_address = Addr::unchecked("seller_address");
        let info = mock_info("seller_address", &[]);
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
                max_bid_count: 2,
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

        save_seller_state(&mut deps.storage, &Seller {
            seller_address,
            accepted_value_cents: Uint128::new(100000000),
            pool_denoms: vec!["mock.denom".to_string()],
            offer_hash: "mock-offer-hash".to_string(),
        }).unwrap();

        save_bid_list_state(&mut deps.storage, &BidList {
            bids: vec![
                Bid {
                    buyer_address: Addr::unchecked("buyer-address-0"),
                    agreement_terms_hash: "mock-hash-buyers-0".to_string(),
                },
                Bid {
                    buyer_address: Addr::unchecked("buyer-address-1"),
                    agreement_terms_hash: "mock-hash-buyers-1".to_string(),
                }
            ],
        }).unwrap();

        save_transaction_state(&mut deps.storage, &TransactionState {
            buyer_address: Addr::unchecked("buyer-address-0"),
            buyer_has_accepted_pools: false,
            agreement_terms_hash: "mock-hash-buyers-0".to_string(),
        }).unwrap();

        let accept_buyer_message = AcceptBid {
            bidder_address: "buyer-address-1".to_string(),
            agreement_terms_hash: "mock-hash-buyers-1".to_string(),
        };
        match execute(deps.as_mut(), env, info, accept_buyer_message) {
            Ok(_) => {
                panic!("failed to detect error when accepting a buyer after a buyer has already been accepted")
            }
            Err(error) => {
                match error {
                    ContractError::BidPreviouslyAccepted { address } => {
                        assert_eq!(address, "buyer-address-0")
                    }
                    _ => {
                        panic!("unexpected error encountered when accepting a buyer after a buyer has already been accepted")
                    }
                }
            }
        }
    }
}