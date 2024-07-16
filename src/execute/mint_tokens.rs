use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, Uint128};
use crate::error::ContractError;
use crate::error::ContractError::{InvalidTokenCount, TokensAlreadyMinted, UnauthorizedToMint};
use crate::storage::state_store::{retrieve_contract_config, retrieve_optional_token_data_state, save_token_data_state, TokenData};
use crate::util::helpers::{create_mint_tokens_messages, is_contract_admin};

pub fn execute_mint_tokens(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_count: Uint128,
    token_denom: String,
) -> Result<Response, ContractError> {
    if !is_contract_admin(&deps, &info)? {
        return Err(UnauthorizedToMint)
    }

    match retrieve_optional_token_data_state(deps.storage)? {
        Some(data) => {
            Err(TokensAlreadyMinted { token_denom: data.token_denom })
        }
        None => {
            // We must mint at least one coin
            if token_count <= Uint128::new(0) {
                return Err(InvalidTokenCount);
            }

            // Save the token denom and amount in the state; the coins will not be transferred
            // until the seller accepts a bid
            save_token_data_state(deps.storage, &TokenData {
                token_denom: token_denom.to_string(),
                token_count: token_count.clone(),
            })?;

            let config = retrieve_contract_config(deps.storage)?;
            Ok(
                Response::new().add_messages(
                    // Mint the coins
                    create_mint_tokens_messages(token_denom, token_count, config.dealers, env.contract.address.to_string())
                )
            )
        }
    }
}
