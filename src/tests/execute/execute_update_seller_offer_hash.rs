#[cfg(test)]
mod execute_update_seller_offer_hash {
    use crate::contract::execute;
    use crate::error::ContractError;
    use crate::msg::ExecuteMsg::{UpdateSellerOfferHash};
    use crate::query::contract_state::query_contract_state;
    use crate::storage::state_store::{save_bid_list_state, save_contract_config, Config, BidList, save_seller_state, Seller};
    use crate::version_info::{set_version_info, VersionInfoV1};
    use cosmwasm_std::testing::{mock_env, mock_info};
    use cosmwasm_std::{Addr, Uint128};
    use provwasm_mocks::mock_provenance_dependencies;

    #[test]
    fn update_seller_offer_hash() {
        let mut deps = mock_provenance_dependencies();
        let info = mock_info("public-seller-0", &[]);
        let env = mock_env();

        save_contract_config(
            &mut deps.storage,
            &Config {
                use_private_sellers: false,
                use_private_buyers: false,
                allowed_sellers: vec![],
                allowed_buyers: vec![],
                token_denom: "test.forward.market.token".to_string(),
                token_count: Uint128::new(1000),
                dealers: vec![Addr::unchecked("dealer-address")],
                is_disabled: false,
                max_bid_count: 3,
                contract_admin: Addr::unchecked("contract-admin")
            },
        ).unwrap();

        save_bid_list_state(&mut deps.storage, &BidList {
            bids: vec![],
        }).unwrap();

        set_version_info(
            &mut deps.storage,
            &VersionInfoV1 {
                definition: "mock".to_string(),
                version: "0.0.0".to_string(),
            },
        ).unwrap();

        save_seller_state(&mut deps.storage, &Seller {
            seller_address: Addr::unchecked("public-seller-0"),
            accepted_value_cents: Uint128::new(200000000),
            pool_denoms: vec!["test.denom.mock".to_string()],
            offer_hash: "to-be-replaced".to_string(),
        }).unwrap();

        let update_hash_message = UpdateSellerOfferHash { offer_hash: "new-hash".to_string() };
        match execute(
            deps.as_mut(),
            env.clone(),
            info,
            update_hash_message,
        ) {
            Ok(_) => {
               assert_eq!(
                   query_contract_state(deps.as_ref()).unwrap().seller.unwrap(),
                   Seller {
                       seller_address: Addr::unchecked("public-seller-0"),
                       accepted_value_cents: Uint128::new(200000000),
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
        let info = mock_info("public-seller-1", &[]);
        let env = mock_env();

        save_contract_config(
            &mut deps.storage,
            &Config {
                use_private_sellers: false,
                use_private_buyers: false,
                allowed_sellers: vec![],
                allowed_buyers: vec![],
                token_denom: "test.forward.market.token".to_string(),
                token_count: Uint128::new(1000),
                dealers: vec![Addr::unchecked("dealer-address")],
                is_disabled: false,
                max_bid_count: 3,
                contract_admin: Addr::unchecked("contract-admin")
            },
        ).unwrap();

        save_bid_list_state(&mut deps.storage, &BidList {
            bids: vec![],
        }).unwrap();

        set_version_info(
            &mut deps.storage,
            &VersionInfoV1 {
                definition: "mock".to_string(),
                version: "0.0.0".to_string(),
            },
        ).unwrap();

        save_seller_state(&mut deps.storage, &Seller {
            seller_address: Addr::unchecked("public-seller-0"),
            accepted_value_cents: Uint128::new(200000000),
            pool_denoms: vec!["test.denom.mock".to_string()],
            offer_hash: "to-be-replaced".to_string(),
        }).unwrap();

        let update_hash_message = UpdateSellerOfferHash { offer_hash: "new-hash".to_string() };
        match execute(
            deps.as_mut(),
            env.clone(),
            info,
            update_hash_message,
        ) {
            Ok(_) => {
                panic!("failed to detect error when updating an offer hash as someone other than the seller")
            }
            Err(error) => {
                match error {
                    ContractError::UnauthorizedAsSeller => {}
                    _ => {
                        panic!("unexpected error encountered when updating the offer hash with someone other than the seller")
                    }
                }
            }
        }
    }
}
