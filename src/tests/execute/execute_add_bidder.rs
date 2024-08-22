#[cfg(test)]
mod execute_add_bidder_tests {
    use crate::contract::execute;
    use crate::error::ContractError;
    use crate::msg::ExecuteMsg::AddBid;
    use crate::query::contract_state::query_contract_state;
    use crate::storage::state_store::{
        save_bid_list_state, save_contract_config, Bid, BidList, Config,
    };
    use crate::version_info::{set_version_info, VersionInfoV1};
    use cosmwasm_std::testing::mock_env;
    use cosmwasm_std::MessageInfo;
    use provwasm_mocks::mock_provenance_dependencies;

    #[test]
    fn add_bidders_to_public_forward_market() {
        let mut deps = mock_provenance_dependencies();
        let bidder_address = deps.api.addr_make("bidder-address");
        let info = MessageInfo {
            sender: bidder_address.clone(),
            funds: vec![],
        };
        let env = mock_env();

        save_contract_config(
            &mut deps.storage,
            &Config {
                use_private_sellers: false,
                use_private_buyers: false,
                allowed_sellers: vec![],
                allowed_buyers: vec![],
                dealers: vec![deps.api.addr_make("dealer-address")],
                is_disabled: false,
                max_bid_count: 3,
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

        let existing_bidder = Bid {
            buyer_address: deps.api.addr_make("existing-buyer-address"),
            agreement_terms_hash: "mock-hash-existing-buyers".to_string(),
        };
        save_bid_list_state(
            &mut deps.storage,
            &BidList {
                bids: vec![existing_bidder.clone()],
            },
        )
        .unwrap();

        let add_bidder_message = AddBid {
            agreement_terms_hash: "buyer-mock-hash".to_string(),
        };
        match execute(deps.as_mut(), env, info, add_bidder_message) {
            Ok(_) => {
                assert_eq!(
                    query_contract_state(deps.as_ref()).unwrap().bids,
                    vec![
                        existing_bidder.clone(),
                        Bid {
                            buyer_address: bidder_address.clone(),
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
        let bidder_address = deps.api.addr_make("bidder-address");
        let info = MessageInfo {
            sender: bidder_address.clone(),
            funds: vec![],
        };
        let env = mock_env();

        save_contract_config(
            &mut deps.storage,
            &Config {
                use_private_sellers: false,
                use_private_buyers: true,
                allowed_sellers: vec![],
                allowed_buyers: vec![bidder_address.clone()],
                dealers: vec![deps.api.addr_make("dealer-address")],
                is_disabled: false,
                max_bid_count: 3,
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

        save_bid_list_state(&mut deps.storage, &BidList { bids: vec![] }).unwrap();

        let add_bidder_message = AddBid {
            agreement_terms_hash: "buyer-mock-hash".to_string(),
        };
        match execute(deps.as_mut(), env, info, add_bidder_message) {
            Ok(_) => {
                assert_eq!(
                    query_contract_state(deps.as_ref()).unwrap().bids,
                    vec![Bid {
                        buyer_address: bidder_address.clone(),
                        agreement_terms_hash: "buyer-mock-hash".to_string(),
                    }]
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
        let bidder_address = deps.api.addr_make("bidder-address");
        let info = MessageInfo {
            sender: bidder_address.clone(),
            funds: vec![],
        };
        let env = mock_env();

        save_contract_config(
            &mut deps.storage,
            &Config {
                use_private_sellers: false,
                use_private_buyers: true,
                allowed_sellers: vec![],
                allowed_buyers: vec![deps.api.addr_make("allowed-bidder-address")],
                dealers: vec![deps.api.addr_make("dealer-address")],
                is_disabled: false,
                max_bid_count: 3,
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

        save_bid_list_state(&mut deps.storage, &BidList { bids: vec![] }).unwrap();

        let add_bidder_message = AddBid {
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
        let bidder_address = deps.api.addr_make("bidder-address");
        let info = MessageInfo {
            sender: bidder_address.clone(),
            funds: vec![],
        };
        let env = mock_env();

        save_contract_config(
            &mut deps.storage,
            &Config {
                use_private_sellers: false,
                use_private_buyers: true,
                allowed_sellers: vec![],
                allowed_buyers: vec![bidder_address.clone()],
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

        save_bid_list_state(
            &mut deps.storage,
            &BidList {
                bids: vec![
                    Bid {
                        buyer_address: deps.api.addr_make("existing-buyer-address-0"),
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

        let add_bidder_message = AddBid {
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
