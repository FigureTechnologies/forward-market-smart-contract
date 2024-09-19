#[cfg(test)]
mod execute_rescind_bid_tests {
    use crate::contract::execute;
    use crate::error::ContractError;
    use crate::msg::ExecuteMsg::{RescindBid};
    use crate::query::contract_state::query_contract_state;
    use crate::storage::state_store::{save_bid_list_state, save_contract_config, Bid, BidList, Config, save_buyer_state, Buyer};
    use crate::version_info::{set_version_info, VersionInfoV1};
    use cosmwasm_std::testing::mock_env;
    use cosmwasm_std::MessageInfo;
    use provwasm_mocks::mock_provenance_dependencies;

    #[test]
    fn rescind_bid_on_forward_market() {
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
            buyer_address: bidder_address,
            agreement_terms_hash: "mock-hash-existing-buyers".to_string(),
        };
        save_bid_list_state(
            &mut deps.storage,
            &BidList {
                bids: vec![existing_bidder.clone()],
            },
        )
        .unwrap();

        match execute(deps.as_mut(), env, info, RescindBid {}) {
            Ok(_) => {
                assert_eq!(
                    query_contract_state(deps.as_ref()).unwrap().bids.len(),
                    0
                );
            }
            Err(error) => {
                panic!("failed to rescind bidder: {:?}", error)
            }
        }
    }

    #[test]
    fn rescind_bid_after_acceptance() {
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
            buyer_address: bidder_address.clone(),
            agreement_terms_hash: "mock-hash-existing-buyers".to_string(),
        };
        save_bid_list_state(
            &mut deps.storage,
            &BidList {
                bids: vec![existing_bidder.clone()],
            },
        )
            .unwrap();

        save_buyer_state(
            &mut deps.storage,
            &Buyer {
                buyer_address: bidder_address.clone(),
                buyer_has_accepted_pools: false,
                agreement_terms_hash: "".to_string(),
            }
        ).unwrap();

        match execute(deps.as_mut(), env, info, RescindBid {}) {
            Ok(_) => {
                assert_eq!(
                    query_contract_state(deps.as_ref()).unwrap().bids.len(),
                    0
                );
            }
            Err(error) => {
                match error {
                    ContractError::IllegalBidRescind => {},
                    _ => {
                        panic!(
                            "unexpected error encountered when attempting to rescind an accepted bid: {:?}",
                            error
                        )
                    }
                }
            }
        }
    }

    #[test]
    fn rescind_bid_after_acceptance_but_not_accepted_bidder() {
        let mut deps = mock_provenance_dependencies();
        let bidder_address_0 = deps.api.addr_make("bidder-address-0");
        let bidder_address_1 = deps.api.addr_make("bidder-address-1");
        let info = MessageInfo {
            sender: bidder_address_0.clone(),
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

        save_bid_list_state(
            &mut deps.storage,
            &BidList {
                bids: vec![
                    Bid {
                        buyer_address: bidder_address_0,
                        agreement_terms_hash: "mock-hash-existing-buyers-0".to_string(),
                    },
                    Bid {
                        buyer_address: bidder_address_1.clone(),
                        agreement_terms_hash: "mock-hash-existing-buyers-1".to_string(),
                    }
                ],
            },
        )
            .unwrap();

        save_buyer_state(
            &mut deps.storage,
            &Buyer {
                buyer_address: bidder_address_1.clone(),
                buyer_has_accepted_pools: false,
                agreement_terms_hash: "".to_string(),
            }
        ).unwrap();

        match execute(deps.as_mut(), env, info, RescindBid {}) {
            Ok(_) => {
                assert_eq!(
                    query_contract_state(deps.as_ref()).unwrap().bids,
                    vec![
                        Bid {
                            buyer_address: bidder_address_1.clone(),
                            agreement_terms_hash: "mock-hash-existing-buyers-1".to_string(),
                        }
                    ]
                );
            }
            Err(error) => {
                panic!("failed to rescind bidder: {:?}", error)
            }
        }
    }
}
