use crate::error::ContractError;
use crate::error::ContractError::InvalidVisibilityConfig;
use crate::storage::state_store::retrieve_contract_config;
use crate::util::helpers::update_config_as_admin;
use cosmwasm_std::{DepsMut, MessageInfo, Response};

pub fn execute_update_allowed_buyers(
    deps: DepsMut,
    info: MessageInfo,
    allowed_buyers: Vec<String>,
) -> Result<Response, ContractError> {
    let config = retrieve_contract_config(deps.storage)?;

    let mut updated_buyers = vec![];
    if config.use_private_buyers{
        for buyer_str in allowed_buyers {
            let seller_addr = deps.api.addr_validate(&buyer_str)?;
            updated_buyers.push(seller_addr)
        }
    } else {
        return Err(InvalidVisibilityConfig);
    }

    let mut updated_config = config.clone();
    updated_config.allowed_buyers = updated_buyers;
    update_config_as_admin(deps, info, updated_config)
}
