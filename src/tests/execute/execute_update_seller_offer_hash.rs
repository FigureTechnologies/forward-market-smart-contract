#[cfg(test)]
mod execute_update_seller_offer_hash {
    use crate::contract::execute;
    use crate::error::ContractError;
    use crate::msg::ExecuteMsg::UpdateSellerOfferHash;
    use crate::query::contract_state::query_contract_state;
    use crate::storage::state_store::{
        save_bid_list_state, save_contract_config, save_seller_state, BidList, Config, Seller,
    };
    use crate::version_info::{set_version_info, VersionInfoV1};
    use cosmwasm_std::testing::mock_env;
    use cosmwasm_std::{MessageInfo};
    use provwasm_mocks::mock_provenance_dependencies;

    #[test]
    fn update_seller_offer_hash() {
        let mut deps = mock_provenance_dependencies();
        let seller_addr = deps.api.addr_make("public-seller-0");
        let info = MessageInfo {
            sender: seller_addr.clone(),
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

        save_bid_list_state(&mut deps.storage, &BidList { bids: vec![] }).unwrap();

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
                seller_address: seller_addr.clone(),
                pool_denoms: vec!["test.denom.mock".to_string()],
                offer_hash: "to-be-replaced".to_string(),
            },
        )
        .unwrap();

        let update_hash_message = UpdateSellerOfferHash {
            offer_hash: "new-hash".to_string(),
        };
        match execute(deps.as_mut(), env.clone(), info, update_hash_message) {
            Ok(_) => {
                assert_eq!(
                    query_contract_state(deps.as_ref()).unwrap().seller.unwrap(),
                    Seller {
                        seller_address: seller_addr.clone(),
                        pool_denoms: vec!["test.denom.mock".to_string()],
                        offer_hash: "new-hash".to_string(),
                    }
                )
            }
            Err(error) => {
                panic!("failed to update seller offer hash: {:?}", error)
            }
        }
    }

    #[test]
    fn update_seller_offer_hash_not_seller() {
        let mut deps = mock_provenance_dependencies();
        let info = MessageInfo {
            sender: deps.api.addr_make("public-seller-1"),
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

        save_bid_list_state(&mut deps.storage, &BidList { bids: vec![] }).unwrap();

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
                seller_address: deps.api.addr_make("public-seller-0"),
                pool_denoms: vec!["test.denom.mock".to_string()],
                offer_hash: "to-be-replaced".to_string(),
            },
        )
        .unwrap();

        let update_hash_message = UpdateSellerOfferHash {
            offer_hash: "new-hash".to_string(),
        };
        match execute(deps.as_mut(), env.clone(), info, update_hash_message) {
            Ok(_) => {
                panic!("failed to detect error when updating an offer hash as someone other than the seller")
            }
            Err(error) => match error {
                ContractError::UnauthorizedAsSeller => {}
                _ => {
                    panic!("unexpected error encountered when updating the offer hash with someone other than the seller")
                }
            },
        }
    }
}
