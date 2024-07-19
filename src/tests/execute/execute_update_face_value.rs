#[cfg(test)]
mod execute_update_face_value {
    use crate::contract::execute;
    use crate::error::ContractError;
    use crate::msg::ExecuteMsg::UpdateFaceValueCents;
    use crate::query::contract_state::query_contract_state;
    use crate::storage::state_store::{save_buyer_state, save_contract_config, Buyer, Config};
    use crate::version_info::{set_version_info, VersionInfoV1};
    use cosmwasm_std::testing::mock_env;
    use cosmwasm_std::{Attribute, MessageInfo, Uint128};
    use provwasm_mocks::mock_provenance_dependencies;

    #[test]
    fn update_face_value_cents() {
        let mut deps = mock_provenance_dependencies();
        let buyer_address = deps.api.addr_make("contract-buyer");
        let seller_address = deps.api.addr_make("allowed-seller-0");
        let dealer_address = deps.api.addr_make("dealer-address");
        let info = MessageInfo {
            sender: buyer_address.clone(),
            funds: vec![],
        };
        let env = mock_env();
        save_contract_config(
            &mut deps.storage,
            &Config {
                is_private: true,
                allowed_sellers: vec![seller_address.clone()],
                agreement_terms_hash: "mock-terms-hash".to_string(),
                token_denom: "test.forward.market.token".into(),
                max_face_value_cents: Uint128::new(9000000000),
                min_face_value_cents: Uint128::new(500000000),
                tick_size: Uint128::new(1000),
                dealers: vec![dealer_address.clone()],
                is_disabled: false,
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

        save_buyer_state(
            &mut deps.storage,
            &Buyer {
                buyer_address: buyer_address.clone(),
                has_accepted_pools: false,
            },
        )
        .unwrap();

        let update_face_value_cents = UpdateFaceValueCents {
            min_face_value_cents: Uint128::new(7500000000),
            max_face_value_cents: Uint128::new(8500000000),
            tick_size: Uint128::new(1000),
        };

        match execute(deps.as_mut(), env, info, update_face_value_cents) {
            Ok(response) => {
                let expected_config_attributes = Config {
                    is_private: true,
                    allowed_sellers: vec![seller_address.clone()],
                    agreement_terms_hash: "mock-terms-hash".to_string(),
                    token_denom: "test.forward.market.token".to_string(),
                    min_face_value_cents: Uint128::new(7500000000),
                    max_face_value_cents: Uint128::new(8500000000),
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
                assert_eq!(
                    query_contract_state(deps.as_ref()).unwrap().config,
                    expected_config_attributes
                );
            }
            Err(error) => {
                panic!("failed to update face value: {:?}", error)
            }
        }
    }

    #[test]
    fn update_face_value_with_unauthorized_buyer() {
        let mut deps = mock_provenance_dependencies();
        let info = MessageInfo {
            sender: deps.api.addr_make("contract-buyer-1"),
            funds: vec![],
        };
        let env = mock_env();
        save_contract_config(
            &mut deps.storage,
            &Config {
                is_private: true,
                allowed_sellers: vec![
                    deps.api.addr_make("allowed-seller-0"),
                    deps.api.addr_make("allowed-seller-1"),
                ],
                agreement_terms_hash: "mock-terms-hash".to_string(),
                token_denom: "test.forward.market.token".into(),
                max_face_value_cents: Uint128::new(500000000),
                min_face_value_cents: Uint128::new(100000000),
                tick_size: Uint128::new(1000),
                dealers: vec![deps.api.addr_make("dealer-address")],
                is_disabled: false,
            },
        )
        .unwrap();

        save_buyer_state(
            &mut deps.storage,
            &Buyer {
                buyer_address: deps.api.addr_make("contract_buyer_0"),
                has_accepted_pools: false,
            },
        )
        .unwrap();

        let update_face_value_cents = UpdateFaceValueCents {
            min_face_value_cents: Uint128::new(300000000),
            max_face_value_cents: Uint128::new(400000000),
            tick_size: Uint128::new(1000),
        };
        match execute(deps.as_mut(), env, info, update_face_value_cents) {
            Ok(_) => {
                panic!("failed to detect an unauthorized buyer when updating the face value")
            }
            Err(error) => match error {
                ContractError::UnauthorizedConfigUpdate => {}
                _ => {
                    panic!(
                        "an unexpected error was returned when attempting to update the face \
                        value with an unauthorized buyer"
                    )
                }
            },
        }
    }
}
