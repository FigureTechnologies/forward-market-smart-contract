#[cfg(test)]
mod execute_update_allowed_sellers {
    use crate::contract::execute;
    use crate::error::ContractError;
    use crate::msg::ExecuteMsg::UpdateAllowedSellers;
    use crate::query::contract_state::query_contract_state;
    use crate::storage::state_store::{save_bid_list_state, save_contract_config, BidList, Config};
    use crate::version_info::{set_version_info, VersionInfoV1};
    use cosmwasm_std::testing::mock_env;
    use cosmwasm_std::{Attribute, MessageInfo};
    use provwasm_mocks::mock_provenance_dependencies;

    #[test]
    fn update_allowed_sellers() {
        let mut deps = mock_provenance_dependencies();
        let buyer_address = deps.api.addr_make("contract-buyer");
        let info = MessageInfo {
            sender: buyer_address.clone(),
            funds: vec![],
        };
        let env = mock_env();
        let allowed_seller_0_addr = deps.api.addr_make("allowed-seller-0");
        let allowed_seller_1_addr = deps.api.addr_make("allowed-seller-1");

        save_contract_config(
            &mut deps.storage,
            &Config {
                use_private_sellers: true,
                use_private_buyers: false,
                allowed_sellers: vec![allowed_seller_0_addr],
                allowed_buyers: vec![],
                dealers: vec![deps.api.addr_make("dealer-address")],
                is_disabled: false,
                max_bid_count: 8,
                contract_admin: info.sender.clone(),
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

        let update_allowed_sellers = UpdateAllowedSellers {
            allowed_sellers: vec![allowed_seller_1_addr.to_string()],
        };

        match execute(deps.as_mut(), env, info.clone(), update_allowed_sellers) {
            Ok(response) => {
                let expected_config_attributes = Config {
                    use_private_sellers: true,
                    use_private_buyers: false,
                    allowed_sellers: vec![allowed_seller_1_addr],
                    allowed_buyers: vec![],
                    dealers: vec![deps.api.addr_make("dealer-address")],
                    is_disabled: false,
                    max_bid_count: 8,
                    contract_admin: info.sender.clone(),
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
            }
            Err(error) => {
                panic!("failed to update allowed sellers: {:?}", error)
            }
        }
    }

    #[test]
    fn update_allowed_sellers_not_private() {
        let mut deps = mock_provenance_dependencies();
        let buyer_address = deps.api.addr_make("contract-buyer");
        let info = MessageInfo {
            sender: buyer_address.clone(),
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
                max_bid_count: 5,
                contract_admin: info.sender.clone(),
            },
        )
        .unwrap();

        let update_allowed_sellers = UpdateAllowedSellers {
            allowed_sellers: vec![deps.api.addr_make("allowed-seller-2").to_string()],
        };
        match execute(deps.as_mut(), env, info, update_allowed_sellers) {
            Ok(_) => {
                panic!(
                    "failed to detect an incorrect configuration when trying to update the \
                    allowed sellers on a non-private contract"
                )
            }
            Err(error) => {
                match error {
                    ContractError::InvalidVisibilityConfig => {}
                    _ => {
                        panic!("an unexpected error was returned when attempting to update the allowed \
                        sellers on a non-private contract")
                    }
                }
            }
        }
    }
}
