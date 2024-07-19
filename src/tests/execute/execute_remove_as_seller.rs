#[cfg(test)]
mod execute_remove_as_seller_tests {
    use crate::contract::execute;
    use crate::error::ContractError;
    use crate::msg::ExecuteMsg::RemoveAsSeller;
    use crate::storage::state_store::{
        save_buyer_state, save_contract_config, save_seller_state, Buyer, Config, Seller,
    };
    use cosmwasm_std::testing::mock_env;
    use cosmwasm_std::{Attribute, MessageInfo, Uint128};
    use provwasm_mocks::mock_provenance_dependencies;

    #[test]
    fn execute_remove_as_seller() {
        let mut deps = mock_provenance_dependencies();
        let seller_address = deps.api.addr_make("seller-0");
        let buyer_address = deps.api.addr_make("contract_buyer");
        let dealer_address = deps.api.addr_make("dealer-address");
        let token_denom = "test.forward.market.token";
        let info = MessageInfo {
            sender: seller_address.clone(),
            funds: vec![],
        };
        let env = mock_env();
        save_contract_config(
            &mut deps.storage,
            &Config {
                is_private: true,
                allowed_sellers: vec![seller_address.clone()],
                agreement_terms_hash: "mock-terms-hash".to_string(),
                token_denom: token_denom.into(),
                max_face_value_cents: Uint128::new(650000000),
                min_face_value_cents: Uint128::new(100000),
                tick_size: Uint128::new(1000),
                dealers: vec![dealer_address.clone()],
                is_disabled: false,
            },
        )
        .unwrap();

        save_buyer_state(
            &mut deps.storage,
            &Buyer {
                buyer_address: buyer_address.clone(),
                has_accepted_pools: false,
            },
        )
        .unwrap();

        match execute(deps.as_mut(), env.clone(), info, RemoveAsSeller {}) {
            Ok(response) => {
                let expected_config_attributes = Config {
                    is_private: true,
                    allowed_sellers: vec![],
                    agreement_terms_hash: "mock-terms-hash".to_string(),
                    token_denom: token_denom.to_string(),
                    min_face_value_cents: Uint128::new(100000),
                    max_face_value_cents: Uint128::new(650000000),
                    tick_size: Uint128::new(1000),
                    dealers: vec![dealer_address.clone()],
                    is_disabled: false,
                };
                assert_eq!(response.attributes.len(), 1);
                assert_eq!(
                    response.attributes[0],
                    Attribute::new(
                        "contract_config",
                        format!("{:?}", expected_config_attributes)
                    )
                );
            }
            Err(error) => {
                panic!(
                    "failed to remove the seller from the allowed seller's list: {:?}",
                    error
                )
            }
        }
    }

    #[test]
    fn execute_remove_as_seller_not_private() {
        let mut deps = mock_provenance_dependencies();
        let seller_address = deps.api.addr_make("allowed-seller-0");
        let buyer_address = deps.api.addr_make("contract_buyer");
        let token_denom = "test.forward.market.token";
        let info = MessageInfo {
            sender: seller_address.clone(),
            funds: vec![],
        };
        let env = mock_env();
        save_contract_config(
            &mut deps.storage,
            &Config {
                is_private: false,
                allowed_sellers: vec![],
                agreement_terms_hash: "mock-terms-hash".to_string(),
                token_denom: token_denom.into(),
                max_face_value_cents: Uint128::new(650000000),
                min_face_value_cents: Uint128::new(100000),
                tick_size: Uint128::new(1000),
                dealers: vec![deps.api.addr_make("dealer-address")],
                is_disabled: false,
            },
        )
        .unwrap();

        save_buyer_state(
            &mut deps.storage,
            &Buyer {
                buyer_address: buyer_address.clone(),
                has_accepted_pools: false,
            },
        )
        .unwrap();

        match execute(deps.as_mut(), env.clone(), info, RemoveAsSeller {}) {
            Ok(_) => {
                panic!("failed to detect error when removing a seller on a public contract")
            }
            Err(error) => match error {
                ContractError::InvalidSellerRemovalRequest => {}
                _ => {
                    panic!(
                        "Unexpected error encountered when attempting to remove a seller \
                        from a public contract"
                    )
                }
            },
        }
    }

    #[test]
    fn execute_remove_as_seller_not_in_list() {
        let mut deps = mock_provenance_dependencies();
        let seller_address = deps.api.addr_make("allowed-seller-0");
        let buyer_address = deps.api.addr_make("contract_buyer");
        let token_denom = "test.forward.market.token";
        let info = MessageInfo {
            sender: seller_address.clone(),
            funds: vec![],
        };
        let env = mock_env();
        save_contract_config(
            &mut deps.storage,
            &Config {
                is_private: true,
                allowed_sellers: vec![deps.api.addr_make("allowed-seller-1")],
                agreement_terms_hash: "mock-terms-hash".to_string(),
                token_denom: token_denom.into(),
                max_face_value_cents: Uint128::new(650000000),
                min_face_value_cents: Uint128::new(100000),
                tick_size: Uint128::new(1000),
                dealers: vec![deps.api.addr_make("dealer-address")],
                is_disabled: false,
            },
        )
        .unwrap();

        save_buyer_state(
            &mut deps.storage,
            &Buyer {
                buyer_address: buyer_address.clone(),
                has_accepted_pools: false,
            },
        )
        .unwrap();

        match execute(deps.as_mut(), env.clone(), info, RemoveAsSeller {}) {
            Ok(_) => {
                panic!(
                    "failed to detect error when removing a seller that is not in the list \
                of allowed sellers"
                )
            }
            Err(error) => match error {
                ContractError::IllegalSellerRemovalRequest => {}
                _ => {
                    panic!(
                        "Unexpected error encountered when attempting to remove a seller \
                        that is not in the list of allowed sellers"
                    )
                }
            },
        }
    }

    #[test]
    fn execute_remove_as_seller_already_accepted() {
        let mut deps = mock_provenance_dependencies();
        let seller_address = deps.api.addr_make("allowed-seller-0");
        let buyer_address = deps.api.addr_make("contract-buyer");
        let token_denom = "test.forward.market.token";
        let info = MessageInfo {
            sender: seller_address.clone(),
            funds: vec![],
        };
        let env = mock_env();
        save_contract_config(
            &mut deps.storage,
            &Config {
                is_private: true,
                allowed_sellers: vec![seller_address.clone()],
                agreement_terms_hash: "mock-terms-hash".to_string(),
                token_denom: token_denom.into(),
                max_face_value_cents: Uint128::new(650000000),
                min_face_value_cents: Uint128::new(500000),
                tick_size: Uint128::new(1000),
                dealers: vec![deps.api.addr_make("dealer-address")],
                is_disabled: false,
            },
        )
        .unwrap();

        save_buyer_state(
            &mut deps.storage,
            &Buyer {
                buyer_address: buyer_address.clone(),
                has_accepted_pools: false,
            },
        )
        .unwrap();

        save_seller_state(
            &mut deps.storage,
            &Seller {
                seller_address: seller_address.clone(),
                accepted_value_cents: Uint128::new(1000000),
                pool_denoms: vec![],
                offer_hash: "mock-offer-hash".to_string(),
            },
        )
        .unwrap();

        match execute(deps.as_mut(), env.clone(), info, RemoveAsSeller {}) {
            Ok(_) => {
                panic!(
                    "failed to detect error when removing a seller that has already accepted \
                the contract"
                )
            }
            Err(error) => match error {
                ContractError::SellerAlreadyAccepted => {}
                _ => {
                    panic!(
                        "Unexpected error encountered when attempting to remove a seller \
                        that has already accepted the contract"
                    )
                }
            },
        }
    }
}
