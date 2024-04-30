use crate::error::ContractError;
use crate::error::ContractError::{
    IllegalSellerRemovalRequest, InvalidSellerRemovalRequest, SellerAlreadyAccepted,
};
use crate::storage::state_store::{retrieve_contract_config, save_contract_config};
use crate::util::helpers::is_seller;
use cosmwasm_std::{DepsMut, MessageInfo, Response};

pub fn execute_remove_as_seller(
    deps: DepsMut,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = retrieve_contract_config(deps.storage)?;
    // If the contract is not private the allowed seller's list is not used
    if !config.is_private {
        return Err(InvalidSellerRemovalRequest);
    }

    // Return an error if the sender isn't in the list they are trying to be removed from
    if !config.allowed_sellers.contains(&info.sender) {
        return Err(IllegalSellerRemovalRequest);
    }

    // Return an error if the seller has already added themselves as the seller of the contract
    if is_seller(&deps, &info)? {
        return Err(SellerAlreadyAccepted);
    }

    let mut updated_config = retrieve_contract_config(deps.storage)?.clone();
    let mut updated_sellers_list = config.allowed_sellers.clone();
    let index_to_remove = config
        .allowed_sellers
        .into_iter()
        .position(|addr| addr == info.sender)
        .unwrap(); // We can use unwrap because existence has been verified above
    updated_sellers_list.remove(index_to_remove);
    updated_config.allowed_sellers = updated_sellers_list;
    save_contract_config(deps.storage, &updated_config)?;

    Ok(Response::new().add_attribute("contract_config", format!("{:?}", updated_config)))
}
