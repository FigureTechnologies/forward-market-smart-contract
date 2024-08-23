use crate::error::ContractError;
use crate::error::ContractError::{SellerAlreadyExists, UnauthorizedPrivateSeller};

use crate::storage::state_store::{
    retrieve_contract_config, retrieve_optional_seller_state, save_seller_state, Seller,
};
use cosmwasm_std::{DepsMut, MessageInfo, Response};

pub fn execute_add_seller(
    deps: DepsMut,
    info: MessageInfo,
    offer_hash: String,
) -> Result<Response, ContractError> {
    // Make sure we haven't already set the seller config. If we have, return an error
    match retrieve_optional_seller_state(deps.storage)? {
        None => {}
        Some(_) => return Err(SellerAlreadyExists),
    }

    let config = retrieve_contract_config(deps.storage)?;

    // If using private sellers, make sure this seller is allowed to engage with the contract
    if config.use_private_sellers && !config.allowed_sellers.contains(&info.sender) {
        return Err(UnauthorizedPrivateSeller);
    }

    // Store the seller information
    let seller_state = Seller {
        seller_address: info.sender.clone(),
        pool_denoms: vec![],
        offer_hash,
    };
    save_seller_state(deps.storage, &seller_state)?;

    Ok(Response::new().add_attribute("seller_state", format!("{:?}", seller_state)))
}
