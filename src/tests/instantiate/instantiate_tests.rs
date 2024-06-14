#[cfg(test)]
mod instantiate_tests {
    use crate::contract::instantiate;
    use crate::error::ContractError;
    use crate::msg::InstantiateContractMsg;
    use crate::query::contract_state::query_contract_state;
    use crate::storage::state_store::{retrieve_buyer_state, Config};
    use crate::version_info::{get_version_info, VersionInfoV1, CRATE_NAME, PACKAGE_VERSION};
    use cosmwasm_std::testing::{mock_env, mock_info};
    use cosmwasm_std::{Addr, Attribute, Storage, Uint128};
    use provwasm_mocks::mock_provenance_dependencies;

    #[test]
    fn instantiate_private_forward_market_contract() {
        let mut deps = mock_provenance_dependencies();
        let info = mock_info("contract-admin", &[]);
        let env = mock_env();
        let instantiate_msg = InstantiateContractMsg {
            use_private_sellers: true,
            use_private_buyers: false,
            allowed_sellers: vec!["allowed-seller-0".into(), "allowed-seller-1".into()],
            allowed_buyers: vec![],
            token_denom: "test.forward.market.token".to_string(),
            min_face_value_cents: Uint128::new(1000000),
            max_face_value_cents: Uint128::new(5000000),
            tick_size: Uint128::new(1000),
            dealers: vec!["dealer-address".to_string()],
            max_buyer_count: 1,
        };
        let init_response = instantiate(deps.as_mut(), env, info, instantiate_msg);
        match init_response {
            Ok(response) => {
                let expected_config_attributes = Config {
                    use_private_sellers: true,
                    use_private_buyers: false,
                    allowed_sellers: vec![
                        Addr::unchecked("allowed-seller-0"),
                        Addr::unchecked("allowed-seller-1"),
                    ],
                    allowed_buyers: vec![],
                    token_denom: "test.forward.market.token".to_string(),
                    min_face_value_cents: Uint128::new(1000000),
                    max_face_value_cents: Uint128::new(5000000),
                    tick_size: Uint128::new(1000),
                    dealers: vec![Addr::unchecked("dealer-address")],
                    is_disabled: false,
                    max_buyer_count: 1,
                    contract_admin: Addr::unchecked("contract-admin")
                };
                assert_eq!(response.attributes.len(), 1);
                assert_eq!(
                    response.attributes[0],
                    Attribute::new(
                        "contract_config",
                        format!("{:?}", expected_config_attributes)
                    )
                );

                assert_eq!(
                    query_contract_state(deps.as_ref()).unwrap().config,
                    expected_config_attributes
                );
                let expected_version_info = VersionInfoV1 {
                    definition: CRATE_NAME.to_string(),
                    version: PACKAGE_VERSION.to_string(),
                };

                assert_eq!(
                    get_version_info(&deps.storage).unwrap(),
                    expected_version_info
                );
            }
            Err(error) => {
                panic!("failed to initialize: {:?}", error)
            }
        }
    }

    #[test]
    fn instantiate_invalid_seller_config() {
        let mut deps = mock_provenance_dependencies();
        let info = mock_info("contract_buyer", &[]);
        let env = mock_env();
        let instantiate_msg = InstantiateContractMsg {
            use_private_sellers: false,
            use_private_buyers: false,
            allowed_sellers: vec!["allowed-seller-0".into(), "allowed-seller-1".into()],
            allowed_buyers: vec![],
            token_denom: "test.forward.market.token".to_string(),
            min_face_value_cents: Uint128::new(1000000),
            max_face_value_cents: Uint128::new(5000000),
            tick_size: Uint128::new(1000),
            dealers: vec!["dealer-address".to_string()],
            max_buyer_count: 1
        };
        let init_response = instantiate(deps.as_mut(), env, info, instantiate_msg);
        match init_response {
            Ok(_) => {
                panic!("failed to detect invalid configuration when seller list is populated but visibility is not \
                private")
            }
            Err(error) => match error {
                ContractError::InvalidVisibilityConfig => {}
                _ => {
                    panic!("returned an unexpected error when seller list is populated but visibility is not private")
                }
            },
        }
    }

    #[test]
    fn instantiate_invalid_face_value() {
        let mut deps = mock_provenance_dependencies();
        let info = mock_info("contract_buyer", &[]);
        let env = mock_env();
        let instantiate_msg = InstantiateContractMsg {
            use_private_sellers: true,
            use_private_buyers: false,
            allowed_sellers: vec!["allowed-seller-0".into(), "allowed-seller-1".into()],
            allowed_buyers: vec![],
            token_denom: "test.forward.market.token".to_string(),
            min_face_value_cents: Uint128::new(0),
            max_face_value_cents: Uint128::new(0),
            tick_size: Uint128::new(0),
            dealers: vec!["dealer-address".to_string()],
            max_buyer_count: 4
        };
        let init_response = instantiate(deps.as_mut(), env, info, instantiate_msg);
        match init_response {
            Ok(_) => {
                panic!("failed to detect invalid face value that is less than 0")
            }
            Err(error) => match error {
                ContractError::FaceValueMustBePositive => {}
                _ => {
                    panic!("returned an unexpected error when supplying an invalid face value")
                }
            },
        }
    }

    #[test]
    fn instantiate_invalid_tick_value() {
        let mut deps = mock_provenance_dependencies();
        let info = mock_info("contract_buyer", &[]);
        let env = mock_env();
        let instantiate_msg = InstantiateContractMsg {
            use_private_sellers: true,
            use_private_buyers: false,
            allowed_sellers: vec!["allowed-seller-0".into(), "allowed-seller-1".into()],
            allowed_buyers: vec![],
            token_denom: "test.forward.market.token".to_string(),
            min_face_value_cents: Uint128::new(10),
            max_face_value_cents: Uint128::new(11),
            tick_size: Uint128::new(5),
            dealers: vec!["dealer-address".to_string()],
            max_buyer_count: 5
        };
        let init_response = instantiate(deps.as_mut(), env, info, instantiate_msg);
        match init_response {
            Ok(_) => {
                panic!("failed to detect invalid tick size that causes face_value_cents / tick_size to have a \
                    remainder")
            }
            Err(error) => match error {
                ContractError::InvalidTickSizeValueMatch => {}
                _ => {
                    panic!("returned an unexpected error when invalid tick size causes face_value_cents / tick_size to \
                        have a remainder")
                }
            },
        }
    }

    #[test]
    fn instantiate_zero_tick_value() {
        let mut deps = mock_provenance_dependencies();
        let info = mock_info("contract_buyer", &[]);
        let env = mock_env();
        let instantiate_msg = InstantiateContractMsg {
            use_private_sellers: true,
            use_private_buyers: false,
            allowed_sellers: vec!["allowed-seller-0".into(), "allowed-seller-1".into()],
            allowed_buyers: vec![],
            token_denom: "test.forward.market.token".to_string(),
            min_face_value_cents: Uint128::new(10),
            max_face_value_cents: Uint128::new(11),
            tick_size: Uint128::new(0),
            dealers: vec!["dealer-address".to_string()],
            max_buyer_count: 10
        };
        let init_response = instantiate(deps.as_mut(), env, info, instantiate_msg);
        match init_response {
            Ok(_) => {
                panic!("failed to detect invalid tick size that causes a division by zero")
            }
            Err(error) => match error {
                ContractError::InvalidTickSizeValueMatch => {}
                _ => {
                    panic!("returned an unexpected error when invalid tick size causes division by zero")
                }
            },
        }
    }
}
