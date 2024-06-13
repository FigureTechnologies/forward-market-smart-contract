#[cfg(test)]
mod execute_update_allowed_sellers {
    use crate::contract::execute;
    use crate::error::ContractError;
    use crate::msg::ExecuteMsg::UpdateAllowedSellers;
    use crate::query::contract_state::query_contract_state;
    use crate::storage::state_store::{save_buyer_state, save_contract_config, Buyer, Config};
    use crate::version_info::{set_version_info, VersionInfoV1};
    use cosmwasm_std::testing::{mock_env, mock_info};
    use cosmwasm_std::{Addr, Attribute, Uint128};
    use provwasm_mocks::mock_provenance_dependencies;

    #[test]
    fn update_allowed_sellers() {
        let mut deps = mock_provenance_dependencies();
        let contract_admin = "contract-admin";
        let info = mock_info(contract_admin, &[]);
        let env = mock_env();
        save_contract_config(
            &mut deps.storage,
            &Config {
                use_private_sellers: true,
                use_private_buyers: false,
                allowed_sellers: vec![Addr::unchecked("allowed-seller-0")],
                allowed_buyers: vec![],
                token_denom: "test.forward.market.token".into(),
                max_face_value_cents: Uint128::new(9000000000),
                min_face_value_cents: Uint128::new(500000000),
                tick_size: Uint128::new(1000),
                dealers: vec![Addr::unchecked("dealer-address")],
                is_disabled: false,
                max_buyer_count: 8,
                contract_admin: info.sender.clone()
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

        let update_allowed_sellers = UpdateAllowedSellers {
            allowed_sellers: vec!["allowed-seller-2".into()],
        };

        match execute(deps.as_mut(), env, info.clone(), update_allowed_sellers) {
            Ok(response) => {
                let expected_config_attributes = Config {
                    use_private_sellers: true,
                    use_private_buyers: false,
                    allowed_sellers: vec![Addr::unchecked("allowed-seller-2")],
                    allowed_buyers: vec![],
                    token_denom: "test.forward.market.token".to_string(),
                    min_face_value_cents: Uint128::new(500000000),
                    max_face_value_cents: Uint128::new(9000000000),
                    tick_size: Uint128::new(1000),
                    dealers: vec![Addr::unchecked("dealer-address")],
                    is_disabled: false,
                    max_buyer_count: 8,
                    contract_admin: info.sender.clone()
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
        let admin_address = "contract-admin";
        let info = mock_info(admin_address, &[]);
        let env = mock_env();
        save_contract_config(
            &mut deps.storage,
            &Config {
                use_private_sellers: false,
                use_private_buyers: false,
                allowed_sellers: vec![],
                allowed_buyers: vec![],
                token_denom: "test.forward.market.token".into(),
                max_face_value_cents: Uint128::new(500000000),
                min_face_value_cents: Uint128::new(200000000),
                tick_size: Uint128::new(1000),
                dealers: vec![Addr::unchecked("dealer-address")],
                is_disabled: false,
                max_buyer_count: 5,
                contract_admin: info.sender.clone()
            },
        )
        .unwrap();

        let update_allowed_sellers = UpdateAllowedSellers {
            allowed_sellers: vec!["allowed-seller-2".into()],
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
