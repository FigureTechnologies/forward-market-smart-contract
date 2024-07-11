#[cfg(test)]
mod execute_add_seller_tests {
    use cosmwasm_std::{Addr, Uint128};
    use cosmwasm_std::testing::{mock_env, mock_info};
    use provwasm_mocks::mock_provenance_dependencies;
    use crate::contract::execute;
    use crate::error::ContractError;
    use crate::msg::ExecuteMsg::MintTokens;
    use crate::storage::state_store::{Config, retrieve_token_data_state, save_contract_config, save_token_data_state, TokenData};

    #[test]
    fn execute_mint_tokens() {
        let mut deps = mock_provenance_dependencies();
        let dealer_address = "dealer-address";
        let seller_address = "allowed-seller-0";
        let admin_address = "contract-admin";
        let info = mock_info(admin_address, &[]);
        let env = mock_env();
        save_contract_config(
            &mut deps.storage,
            &Config {
                use_private_sellers: true,
                use_private_buyers: false,
                allowed_sellers: vec![Addr::unchecked(seller_address)],
                allowed_buyers: vec![],
                dealers: vec![Addr::unchecked(dealer_address)],
                is_disabled: false,
                max_bid_count: 3,
                contract_admin: Addr::unchecked(admin_address),
            },
        ).unwrap();

        match execute(
            deps.as_mut(),
            env.clone(),
            info,
            MintTokens { token_denom: "test.denom.fm".to_string(), token_count: Uint128::new(100) },
        ) {
            Ok(_) => {
                let token_data = retrieve_token_data_state(&mut deps.storage).unwrap();
                assert_eq!(
                    token_data,
                    TokenData {
                        token_denom: "test.denom.fm".to_string(),
                        token_count: Uint128::new(100)
                    }
                )
            }
            Err(error) => {
                panic!("failed to mint tokens: {:?}", error)
            }
        }
    }

    #[test]
    fn execute_mint_tokens_not_admin() {
        let mut deps = mock_provenance_dependencies();
        let dealer_address = "dealer-address";
        let seller_address = "allowed-seller-0";
        let admin_address = "contract-admin";
        let info = mock_info(seller_address, &[]);
        let env = mock_env();
        save_contract_config(
            &mut deps.storage,
            &Config {
                use_private_sellers: true,
                use_private_buyers: false,
                allowed_sellers: vec![Addr::unchecked(seller_address)],
                allowed_buyers: vec![],
                dealers: vec![Addr::unchecked(dealer_address)],
                is_disabled: false,
                max_bid_count: 3,
                contract_admin: Addr::unchecked(admin_address),
            },
        ).unwrap();

        match execute(
            deps.as_mut(),
            env.clone(),
            info,
            MintTokens { token_denom: "test.denom.fm".to_string(), token_count: Uint128::new(100) },
        ) {
            Ok(_) => {
                panic!("failed to detect error when minting coins as non-admin")
            }
            Err(error) => {
                match error {
                    ContractError::UnauthorizedToMint => {},
                    error => {
                        panic!("unexpected error encountered when minting tokens: {:?}", error)
                    }
                }
            }
        }
    }

    #[test]
    fn execute_mint_tokens_already_minted() {
        let mut deps = mock_provenance_dependencies();
        let dealer_address = "dealer-address";
        let seller_address = "allowed-seller-0";
        let admin_address = "contract-admin";
        let info = mock_info(admin_address, &[]);
        let env = mock_env();
        save_contract_config(
            &mut deps.storage,
            &Config {
                use_private_sellers: true,
                use_private_buyers: false,
                allowed_sellers: vec![Addr::unchecked(seller_address)],
                allowed_buyers: vec![],
                dealers: vec![Addr::unchecked(dealer_address)],
                is_disabled: false,
                max_bid_count: 3,
                contract_admin: Addr::unchecked(admin_address),
            },
        ).unwrap();

        save_token_data_state(&mut deps.storage, &TokenData {
            token_denom: "test.denom.fm".to_string(), token_count: Uint128::new(100)
        }).unwrap();

        match execute(
            deps.as_mut(),
            env.clone(),
            info,
            MintTokens { token_denom: "test.denom.fm".to_string(), token_count: Uint128::new(100) },
        ) {
            Ok(_) => {
                panic!("failed to detect error when trying to mint coins after they were already minted")
            }
            Err(error) => {
                match error {
                    ContractError::TokensAlreadyMinted { token_denom: _ } => {},
                    error => {
                        panic!("unexpected error encountered when trying to mint coins after they were already minted: {:?}", error)
                    }
                }
            }
        }
    }
}
