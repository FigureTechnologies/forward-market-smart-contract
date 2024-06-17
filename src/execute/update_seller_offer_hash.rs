use cosmwasm_std::{DepsMut, MessageInfo, Response};
use crate::error::ContractError;
use crate::error::ContractError::UnauthorizedAsSeller;
use crate::storage::state_store::{retrieve_seller_state, save_seller_state};
use crate::util::helpers::is_seller;

pub fn execute_update_seller_offer_hash(
    deps: DepsMut,
    info: MessageInfo,
    offer_hash: String,
) -> Result<Response, ContractError> {
    // Only the seller can update the offer hash
    if !is_seller(&deps, &info)? {
        return Err(UnauthorizedAsSeller);
    }

    let mut seller_state = retrieve_seller_state(deps.storage)?;
    seller_state.offer_hash = offer_hash;
    save_seller_state(deps.storage, &seller_state)?;
    Ok(Response::new().add_attribute("seller_state", format!("{:?}", seller_state)))
}
