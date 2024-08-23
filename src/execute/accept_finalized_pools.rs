use crate::error::ContractError;
use crate::error::ContractError::{
    IllegalAcceptingParty, IllegalPoolAcceptanceRequest, InvalidOfferHash, PoolAlreadyAccepted,
};
use crate::storage::state_store::{
    retrieve_optional_buyer_state, retrieve_seller_state, save_buyer_state,
};
use crate::util::helpers::{buyer_has_accepted, is_buyer, seller_has_finalized};
use cosmwasm_std::{DepsMut, MessageInfo, Response};

pub fn execute_accept_finalized_pools(
    deps: DepsMut,
    info: MessageInfo,
    offer_hash: String,
) -> Result<Response, ContractError> {
    // Only the buyer can accept the finalized pool list
    if !is_buyer(&deps, &info)? {
        return Err(IllegalAcceptingParty);
    }

    // If the seller hasn't finalized yet, the buyer can't accept
    if !seller_has_finalized(&deps)? {
        return Err(IllegalPoolAcceptanceRequest);
    }

    // If the buyer has already accepted the pool return an error
    if buyer_has_accepted(&deps)? {
        return Err(PoolAlreadyAccepted);
    }

    let mut current_buyer = retrieve_optional_buyer_state(deps.storage)?.unwrap();

    let seller_state = retrieve_seller_state(deps.storage)?;

    if offer_hash != seller_state.offer_hash {
        return Err(InvalidOfferHash);
    }

    current_buyer.buyer_has_accepted_pools = true;

    // Update the transaction state to reflect that the buyer has accepted
    save_buyer_state(deps.storage, &current_buyer)?;

    Ok(Response::new().add_attribute("buyer", format!("{:?}", current_buyer)))
}
