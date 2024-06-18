#[cfg(test)]
mod execute_remove_as_seller_tests {
    use crate::contract::execute;
    use crate::error::ContractError;
    use crate::msg::ExecuteMsg::RemoveAsSeller;
    use crate::storage::state_store::{save_contract_config, save_seller_state, Config, Seller};
    use cosmwasm_std::testing::{mock_env, mock_info};
    use cosmwasm_std::{Addr, Attribute, Uint128};
    use provwasm_mocks::mock_provenance_dependencies;

    #[test]
    fn execute_remove_as_seller() {
        let mut deps = mock_provenance_dependencies();
        let seller_address = "seller-0";
        let token_denom = "test.forward.market.token";
        let info = mock_info(seller_address, &[]);
        let env = mock_env();
        save_contract_config(
            &mut deps.storage,
            &Config {
                use_private_sellers: true,
                use_private_buyers: false,
                allowed_sellers: vec![Addr::unchecked(seller_address)],
                allowed_buyers: vec![],
                token_denom: token_denom.into(),
                token_count: Uint128::new(1000),
                dealers: vec![Addr::unchecked("dealer-address")],
                is_disabled: false,
                max_bid_count: 5,
                contract_admin: Addr::unchecked("contract-admin"),
            },
        )
        .unwrap();

        match execute(deps.as_mut(), env.clone(), info, RemoveAsSeller {}) {
            Ok(response) => {
                let expected_config_attributes = Config {
                    use_private_sellers: true,
                    use_private_buyers: false,
                    allowed_sellers: vec![],
                    allowed_buyers: vec![],
                    token_denom: token_denom.to_string(),
                    token_count: Uint128::new(1000),
                    dealers: vec![Addr::unchecked("dealer-address")],
                    is_disabled: false,
                    max_bid_count: 5,
                    contract_admin: Addr::unchecked("contract-admin"),
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
        let seller_address = "allowed-seller-0";
        let token_denom = "test.forward.market.token";
        let info = mock_info(seller_address, &[]);
        let env = mock_env();
        save_contract_config(
            &mut deps.storage,
            &Config {
                use_private_sellers: false,
                use_private_buyers: false,
                allowed_sellers: vec![],
                allowed_buyers: vec![],
                token_denom: token_denom.into(),
                token_count: Uint128::new(1000),
                dealers: vec![Addr::unchecked("dealer-address")],
                is_disabled: false,
                max_bid_count: 3,
                contract_admin: Addr::unchecked("contract-admin"),
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
        let seller_address = "allowed-seller-0";
        let token_denom = "test.forward.market.token";
        let info = mock_info(seller_address, &[]);
        let env = mock_env();
        save_contract_config(
            &mut deps.storage,
            &Config {
                use_private_sellers: true,
                use_private_buyers: false,
                allowed_sellers: vec![Addr::unchecked("allowed-seller-1")],
                allowed_buyers: vec![],
                token_denom: token_denom.into(),
                token_count: Uint128::new(1000),
                dealers: vec![Addr::unchecked("dealer-address")],
                is_disabled: false,
                max_bid_count: 2,
                contract_admin: Addr::unchecked("contract-admin"),
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
        let seller_address = "allowed-seller-0";
        let token_denom = "test.forward.market.token";
        let info = mock_info(seller_address, &[]);
        let env = mock_env();
        save_contract_config(
            &mut deps.storage,
            &Config {
                use_private_sellers: true,
                use_private_buyers: true,
                allowed_sellers: vec![Addr::unchecked(seller_address)],
                allowed_buyers: vec![],
                token_denom: token_denom.into(),
                token_count: Uint128::new(1000),
                dealers: vec![Addr::unchecked("dealer-address")],
                is_disabled: false,
                max_bid_count: 10,
                contract_admin: Addr::unchecked("contract-admin"),
            },
        )
        .unwrap();

        save_seller_state(
            &mut deps.storage,
            &Seller {
                seller_address: Addr::unchecked(seller_address),
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
