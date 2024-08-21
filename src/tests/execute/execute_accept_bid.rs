#[cfg(test)]
mod execute_accept_bid_tests {
    use crate::contract::execute;
    use crate::error::ContractError;
    use crate::msg::ExecuteMsg::AcceptBid;
    use crate::query::contract_state::query_contract_state;
    use crate::storage::state_store::{
        save_bid_list_state, save_buyer_state, save_contract_config, save_seller_state,
        save_token_data_state, Bid, BidList, Buyer, Config, Seller, TokenData,
    };
    use crate::version_info::{set_version_info, VersionInfoV1};
    use cosmwasm_std::testing::mock_env;
    use cosmwasm_std::{MessageInfo, Uint128};
    use provwasm_mocks::mock_provenance_dependencies;

    #[test]
    fn accept_bid() {
        let mut deps = mock_provenance_dependencies();
        let seller_address = deps.api.addr_make("seller_address");
        let info = MessageInfo {
            sender: seller_address.clone(),
            funds: vec![],
        };
        let env = mock_env();

        save_contract_config(
            &mut deps.storage,
            &Config {
                use_private_sellers: false,
                use_private_buyers: true,
                allowed_sellers: vec![],
                allowed_buyers: vec![deps.api.addr_make("bidder_address")],
                dealers: vec![deps.api.addr_make("dealer-address")],
                is_disabled: false,
                max_bid_count: 2,
                contract_admin: deps.api.addr_make("contract-admin"),
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

        save_seller_state(
            &mut deps.storage,
            &Seller {
                seller_address,
                pool_denoms: vec!["mock.denom".to_string()],
                offer_hash: "mock-offer-hash".to_string(),
            },
        )
        .unwrap();

        let existing_buyer_0_address = deps.api.addr_make("existing-buyer-address-0");
        save_bid_list_state(
            &mut deps.storage,
            &BidList {
                bids: vec![
                    Bid {
                        buyer_address: existing_buyer_0_address.clone(),
                        agreement_terms_hash: "mock-hash-existing-buyers-0".to_string(),
                    },
                    Bid {
                        buyer_address: deps.api.addr_make("existing-buyer-address-1"),
                        agreement_terms_hash: "mock-hash-existing-buyers-1".to_string(),
                    },
                ],
            },
        )
        .unwrap();

        save_token_data_state(
            &mut deps.storage,
            &TokenData {
                token_denom: "test.token.fm".to_string(),
                token_count: Uint128::new(10),
            },
        )
        .unwrap();

        let accept_bid_message = AcceptBid {
            bidder_address: existing_buyer_0_address.clone().to_string(),
            agreement_terms_hash: "mock-hash-existing-buyers-0".to_string(),
        };
        match execute(deps.as_mut(), env, info, accept_bid_message) {
            Ok(_) => {
                assert_eq!(
                    query_contract_state(deps.as_ref()).unwrap().buyer.unwrap(),
                    Buyer {
                        buyer_address: deps.api.addr_make("existing-buyer-address-0"),
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
        let seller_address = deps.api.addr_make("seller_address");
        let info = MessageInfo {
            sender: seller_address.clone(),
            funds: vec![],
        };
        let env = mock_env();

        save_contract_config(
            &mut deps.storage,
            &Config {
                use_private_sellers: false,
                use_private_buyers: true,
                allowed_sellers: vec![],
                allowed_buyers: vec![deps.api.addr_make("bidder-address")],
                dealers: vec![deps.api.addr_make("dealer-address")],
                is_disabled: false,
                max_bid_count: 2,
                contract_admin: deps.api.addr_make("contract-admin"),
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

        save_seller_state(
            &mut deps.storage,
            &Seller {
                seller_address,
                pool_denoms: vec!["mock.denom".to_string()],
                offer_hash: "mock-offer-hash".to_string(),
            },
        )
        .unwrap();

        save_bid_list_state(
            &mut deps.storage,
            &BidList {
                bids: vec![Bid {
                    buyer_address: deps.api.addr_make("existing-bidder-address-0"),
                    agreement_terms_hash: "mock-hash-existing-buyers-0".to_string(),
                }],
            },
        )
        .unwrap();

        save_token_data_state(
            &mut deps.storage,
            &TokenData {
                token_denom: "test.token.fm".to_string(),
                token_count: Uint128::new(10),
            },
        )
        .unwrap();

        let accept_buyer_message = AcceptBid {
            bidder_address: deps
                .api
                .addr_make("non-existing-bidder-address")
                .to_string(),
            agreement_terms_hash: "mock-hash".to_string(),
        };
        match execute(deps.as_mut(), env, info, accept_buyer_message) {
            Ok(_) => {
                panic!("failed to detect error when accepting a buyer with an address not in the bidder list")
            }
            Err(error) => match error {
                ContractError::BidDoesNotExist { address } => {
                    assert_eq!(
                        address,
                        deps.api
                            .addr_make("non-existing-bidder-address")
                            .to_string()
                    )
                }
                _ => {
                    panic!("unexpected error encountered when accepting a buyer with an address not in the bidder list")
                }
            },
        }
    }

    #[test]
    fn accept_buyer_with_invalid_hash() {
        let mut deps = mock_provenance_dependencies();
        let seller_address = deps.api.addr_make("seller_address");
        let info = MessageInfo {
            sender: seller_address.clone(),
            funds: vec![],
        };
        let env = mock_env();

        save_contract_config(
            &mut deps.storage,
            &Config {
                use_private_sellers: false,
                use_private_buyers: true,
                allowed_sellers: vec![],
                allowed_buyers: vec![deps.api.addr_make("bidder_address")],
                dealers: vec![deps.api.addr_make("dealer-address")],
                is_disabled: false,
                max_bid_count: 2,
                contract_admin: deps.api.addr_make("contract-admin"),
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

        save_seller_state(
            &mut deps.storage,
            &Seller {
                seller_address,
                pool_denoms: vec!["mock.denom".to_string()],
                offer_hash: "mock-offer-hash".to_string(),
            },
        )
        .unwrap();

        save_bid_list_state(
            &mut deps.storage,
            &BidList {
                bids: vec![Bid {
                    buyer_address: deps.api.addr_make("existing-buyer-address-0"),
                    agreement_terms_hash: "mock-hash-existing-buyers-0".to_string(),
                }],
            },
        )
        .unwrap();

        save_token_data_state(
            &mut deps.storage,
            &TokenData {
                token_denom: "test.token.fm".to_string(),
                token_count: Uint128::new(10),
            },
        )
        .unwrap();

        let accept_buyer_message = AcceptBid {
            bidder_address: deps.api.addr_make("existing-buyer-address-0").to_string(),
            agreement_terms_hash: "stale-hash".to_string(),
        };
        match execute(deps.as_mut(), env, info, accept_buyer_message) {
            Ok(_) => {
                panic!("failed to detect error when accepting a buyer with an invalid agreement terms hash")
            }
            Err(error) => match error {
                ContractError::InvalidAgreementTermsHash => {}
                _ => {
                    panic!("unexpected error encountered when accepting a buyer with an invalid agreement terms hash")
                }
            },
        }
    }

    #[test]
    fn accept_buyer_when_buyer_previously_accepted() {
        let mut deps = mock_provenance_dependencies();
        let seller_address = deps.api.addr_make("seller_address");
        let buyer_address_0 = deps.api.addr_make("buyer-address-0");
        let buyer_address_1 = deps.api.addr_make("buyer-address-1");
        let info = MessageInfo {
            sender: seller_address.clone(),
            funds: vec![],
        };
        let env = mock_env();

        save_contract_config(
            &mut deps.storage,
            &Config {
                use_private_sellers: false,
                use_private_buyers: true,
                allowed_sellers: vec![],
                allowed_buyers: vec![deps.api.addr_make("bidder-address")],
                dealers: vec![deps.api.addr_make("dealer-address")],
                is_disabled: false,
                max_bid_count: 2,
                contract_admin: deps.api.addr_make("contract-admin"),
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

        save_seller_state(
            &mut deps.storage,
            &Seller {
                seller_address,
                pool_denoms: vec!["mock.denom".to_string()],
                offer_hash: "mock-offer-hash".to_string(),
            },
        )
        .unwrap();

        save_bid_list_state(
            &mut deps.storage,
            &BidList {
                bids: vec![
                    Bid {
                        buyer_address: buyer_address_0.clone(),
                        agreement_terms_hash: "mock-hash-buyers-0".to_string(),
                    },
                    Bid {
                        buyer_address: buyer_address_1.clone(),
                        agreement_terms_hash: "mock-hash-buyers-1".to_string(),
                    },
                ],
            },
        )
        .unwrap();

        save_buyer_state(
            &mut deps.storage,
            &Buyer {
                buyer_address: buyer_address_0.clone(),
                buyer_has_accepted_pools: false,
                agreement_terms_hash: "mock-hash-buyers-0".to_string(),
            },
        )
        .unwrap();

        save_token_data_state(
            &mut deps.storage,
            &TokenData {
                token_denom: "test.token.fm".to_string(),
                token_count: Uint128::new(10),
            },
        )
        .unwrap();

        let accept_buyer_message = AcceptBid {
            bidder_address: buyer_address_1.to_string(),
            agreement_terms_hash: "mock-hash-buyers-1".to_string(),
        };
        match execute(deps.as_mut(), env, info, accept_buyer_message) {
            Ok(_) => {
                panic!("failed to detect error when accepting a buyer after a buyer has already been accepted")
            }
            Err(error) => match error {
                ContractError::BidPreviouslyAccepted { address } => {
                    assert_eq!(address, buyer_address_0.to_string())
                }
                _ => {
                    panic!("unexpected error encountered when accepting a buyer after a buyer has already been accepted")
                }
            },
        }
    }
}
