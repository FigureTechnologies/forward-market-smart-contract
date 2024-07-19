#[cfg(test)]
mod instantiate_tests {
    use crate::contract::instantiate;
    use crate::error::ContractError;
    use crate::msg::InstantiateContractMsg;
    use crate::query::contract_state::query_contract_state;
    use crate::storage::state_store::Config;
    use crate::version_info::{get_version_info, VersionInfoV1, CRATE_NAME, PACKAGE_VERSION};
    use cosmwasm_std::testing::mock_env;
    use cosmwasm_std::{Addr, Attribute, MessageInfo, Storage, Uint128};
    use provwasm_mocks::mock_provenance_dependencies;

    #[test]
    fn instantiate_private_forward_market_contract() {
        let mut deps = mock_provenance_dependencies();
        let buyer_address = deps.api.addr_make("contract-buyer");
        let seller_address_0 = deps.api.addr_make("allowed-seller-0");
        let seller_address_1 = deps.api.addr_make("allowed-seller-1");
        let dealer_address = deps.api.addr_make("dealer-address");
        let info = MessageInfo {
            sender: buyer_address.clone(),
            funds: vec![],
        };
        let env = mock_env();
        let instantiate_msg = InstantiateContractMsg {
            use_private_sellers: true,
            use_private_buyers: false,
            allowed_sellers: vec![seller_address_0.clone().to_string(), seller_address_1.clone().to_string()],
            allowed_buyers: vec![],
            dealers: vec!["dealer-address".to_string()],
            max_buyer_count: 1,
        };
        let init_response = instantiate(deps.as_mut(), env, info, instantiate_msg);
        match init_response {
            Ok(response) => {
                let expected_config_attributes = Config {
                    use_private_sellers: true,
                    use_private_buyers: false,
                    allowed_sellers: vec![seller_address_0.clone(), seller_address_1.clone()],
                    allowed_buyers: vec![],
                    dealers: vec![dealer_address.clone()],
                    is_disabled: false,
                    max_bid_count: 1,
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
        let buyer_address = deps.api.addr_make("contract-buyer");
        let info = MessageInfo {
            sender: buyer_address.clone(),
            funds: vec![],
        };
        let env = mock_env();
        let instantiate_msg = InstantiateContractMsg {
            use_private_sellers: false,
            use_private_buyers: false,
            allowed_sellers: vec![
                deps.api.addr_make("allowed-seller-0").to_string(),
                deps.api.addr_make("allowed-seller-1").to_string(),
            ],
            allowed_buyers: vec![],
            dealers: vec![deps.api.addr_make("dealer-address").to_string()],
            max_buyer_count: 1,
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
}
