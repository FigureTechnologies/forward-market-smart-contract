use crate::error::ContractError;
use crate::error::ContractError::{IllegalDisableRequest, UnauthorizedDisableRequest};
use crate::storage::state_store::{retrieve_contract_config, save_contract_config};
use crate::util::helpers::{is_buyer, is_dealer, seller_has_finalized};
use cosmwasm_std::{DepsMut, MessageInfo, Response};

pub fn execute_disable_contract(
    deps: DepsMut,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    // If the seller has finalized, the contract cannot be disabled because the contract
    // is holding the seller's coin(s) so the seller must rescind or the dealer must
    // reset before the contract can be closed
    if seller_has_finalized(&deps)? {
        return Err(IllegalDisableRequest);
    }

    // In order to disable the contract you must be either the buyer or a dealer
    if !is_buyer(&deps, &info)? && !is_dealer(&deps, &info)? {
        return Err(UnauthorizedDisableRequest);
    }

    // Contract is ok to disable, set the flag
    let mut updated_contract_config = retrieve_contract_config(deps.storage)?;
    updated_contract_config.is_disabled = true;
    save_contract_config(deps.storage, &updated_contract_config)?;

    Ok(Response::new().add_attribute("contract_config", format!("{:?}", updated_contract_config)))
}
