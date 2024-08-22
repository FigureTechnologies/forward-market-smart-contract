#[cfg(test)]
mod execute_add_seller_tests {
    use crate::contract::execute;
    use crate::error::ContractError;
    use crate::msg::ExecuteMsg::AddSeller;
    use crate::query::contract_state::query_contract_state;
    use crate::storage::state_store::{
        save_bid_list_state, save_contract_config, save_seller_state, Bid, BidList, Config, Seller,
    };
    use crate::version_info::{set_version_info, VersionInfoV1};
    use cosmwasm_std::testing::mock_env;
    use cosmwasm_std::{MessageInfo};
    use provwasm_mocks::mock_provenance_dependencies;

    #[test]
    fn add_seller_to_public_forward_market() {
        let mut deps = mock_provenance_dependencies();
        let seller_address = deps.api.addr_make("private-seller-0");
        let info = MessageInfo {
            sender: seller_address.clone(),
            funds: vec![],
        };
        let env = mock_env();
        let dealer_address = deps.api.addr_make("dealer_address");
        let buyer_address = deps.api.addr_make("buyer_address");

        let add_seller_msg = AddSeller {
            offer_hash: "mock-offer-hash".to_string(),
        };

        save_contract_config(
            &mut deps.storage,
            &Config {
                use_private_sellers: false,
                use_private_buyers: false,
                allowed_sellers: vec![],
                allowed_buyers: vec![],
                dealers: vec![dealer_address.clone()],
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
                bids: vec![Bid {
                    buyer_address: buyer_address.clone(),
                    agreement_terms_hash: "".to_string(),
                }],
            },
        )
        .unwrap();

        match execute(deps.as_mut(), env, info, add_seller_msg) {
            Ok(_) => {
                assert_eq!(
                    query_contract_state(deps.as_ref()).unwrap().seller.unwrap(),
                    Seller {
                        seller_address: seller_address.clone(),
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
    fn add_duplicate_seller() {
        let mut deps = mock_provenance_dependencies();
        let contract_admin = deps.api.addr_make("contract-admin");
        let info = MessageInfo {
            sender: contract_admin.clone(),
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
                max_bid_count: 2,
                contract_admin: contract_admin.clone(),
            },
        )
        .unwrap();

        save_seller_state(
            &mut deps.storage,
            &Seller {
                seller_address: deps.api.addr_make("existing_seller"),
                pool_denoms: vec![],
                offer_hash: "mock-offer-hash".to_string(),
            },
        )
        .unwrap();

        let add_seller_msg = AddSeller {
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
        let seller_address = deps.api.addr_make("private-seller-0");
        let info = MessageInfo {
            sender: seller_address.clone(),
            funds: vec![],
        };
        let buyer_address = deps.api.addr_make("contract-buyer");
        let env = mock_env();
        let dealer_address = "dealer-address";
        let add_seller_msg = AddSeller {
            offer_hash: "mock-offer-hash".to_string(),
        };

        save_contract_config(
            &mut deps.storage,
            &Config {
                use_private_sellers: true,
                use_private_buyers: true,
                allowed_sellers: vec![seller_address.clone()],
                allowed_buyers: vec![],
                dealers: vec![deps.api.addr_make(dealer_address)],
                is_disabled: false,
                max_bid_count: 2,
                contract_admin: deps.api.addr_make(contract_admin),
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
                    buyer_address: buyer_address.clone(),
                    agreement_terms_hash: "".to_string(),
                }],
            },
        )
        .unwrap();

        match execute(deps.as_mut(), env, info, add_seller_msg) {
            Ok(_) => {
                assert_eq!(
                    query_contract_state(deps.as_ref()).unwrap().seller.unwrap(),
                    Seller {
                        seller_address: seller_address.clone(),
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
        let contract_admin = deps.api.addr_make("contract-admin");
        let info = MessageInfo {
            sender: contract_admin.clone(),
            funds: vec![],
        };
        let env = mock_env();
        let add_seller_msg = AddSeller {
            offer_hash: "mock-offer-hash".to_string(),
        };

        save_contract_config(
            &mut deps.storage,
            &Config {
                use_private_sellers: true,
                use_private_buyers: false,
                allowed_sellers: vec![],
                allowed_buyers: vec![],
                dealers: vec![deps.api.addr_make("dealer-address")],
                is_disabled: false,
                max_bid_count: 5,
                contract_admin: contract_admin.clone(),
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
}
