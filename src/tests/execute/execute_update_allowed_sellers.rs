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
    use crate::msg::ConfigResponse;

    #[test]
    fn update_allowed_sellers() {
        let mut deps = mock_provenance_dependencies();
        let buyer_address = "contract-buyer";
        let info = mock_info(buyer_address, &[]);
        let env = mock_env();
        save_contract_config(
            &mut deps.storage,
            &Config {
                is_private: true,
                allowed_sellers: vec![Addr::unchecked("allowed-seller-0")],
                agreement_terms_hash: "mock-terms-hash".to_string(),
                token_denom: "test.forward.market.token".into(),
                max_face_value_cents: Uint128::new(9000000000),
                min_face_value_cents: Uint128::new(500000000),
                tick_size: Uint128::new(1000),
                dealers: vec![Addr::unchecked("dealer-address")],
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
                buyer_address: Addr::unchecked("contract-buyer"),
                has_accepted_pools: false,
            },
        )
        .unwrap();

        let update_allowed_sellers = UpdateAllowedSellers {
            allowed_sellers: vec!["allowed-seller-2".into()],
        };

        match execute(deps.as_mut(), env, info, update_allowed_sellers) {
            Ok(response) => {
                let expected_config_attributes = ConfigResponse {
                    is_private: true,
                    allowed_sellers: vec![Addr::unchecked("allowed-seller-2")],
                    agreement_terms_hash: "mock-terms-hash".to_string(),
                    token_denom: "test.forward.market.token".to_string(),
                    min_face_value_cents: Uint128::new(500000000),
                    max_face_value_cents: Uint128::new(9000000000),
                    tick_size: Uint128::new(1000),
                    dealers: vec![Addr::unchecked("dealer-address")],
                    is_disabled: false,
                };
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
        let buyer_address = "contract-buyer";
        let info = mock_info(buyer_address, &[]);
        let env = mock_env();
        save_contract_config(
            &mut deps.storage,
            &Config {
                is_private: false,
                allowed_sellers: vec![],
                agreement_terms_hash: "mock-terms-hash".to_string(),
                token_denom: "test.forward.market.token".into(),
                max_face_value_cents: Uint128::new(500000000),
                min_face_value_cents: Uint128::new(200000000),
                tick_size: Uint128::new(1000),
                dealers: vec![Addr::unchecked("dealer-address")],
                is_disabled: false,
            },
        )
        .unwrap();

        save_buyer_state(
            &mut deps.storage,
            &Buyer {
                buyer_address: Addr::unchecked("contract_buyer"),
                has_accepted_pools: false,
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
