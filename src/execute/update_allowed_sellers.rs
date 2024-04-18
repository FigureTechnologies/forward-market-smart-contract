use crate::error::ContractError;
use crate::error::ContractError::InvalidVisibilityConfig;
use crate::storage::state_store::retrieve_contract_config;
use crate::util::helpers::update_config_as_buyer;
use cosmwasm_std::{DepsMut, MessageInfo, Response};

pub fn execute_update_allowed_sellers(
    deps: DepsMut,
    info: MessageInfo,
    allowed_sellers: Vec<String>,
) -> Result<Response, ContractError> {
    let config = retrieve_contract_config(deps.storage)?;

    let mut updated_sellers = vec![];
    if config.is_private {
        for seller_str in allowed_sellers {
            let seller_addr = deps.api.addr_validate(&seller_str)?;
            updated_sellers.push(seller_addr)
        }
    } else {
        return Err(InvalidVisibilityConfig);
    }

    let mut updated_config = config.clone();
    updated_config.allowed_sellers = updated_sellers;
    update_config_as_buyer(deps, info, updated_config)
}
