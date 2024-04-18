use crate::error::ContractError;
use crate::error::ContractError::{
    IllegalRescindRequest, InvalidRescindRequest, UnauthorizedAsSeller,
};
use crate::storage::state_store::{retrieve_seller_state, save_seller_state};
use crate::util::helpers::{
    buyer_has_accepted, create_send_coin_back_to_seller_messages, is_seller,
};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

pub fn execute_rescind_finalized_pools(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    // Only the seller is allowed to rescind the finalized offer
    if !is_seller(&deps, &info)? {
        return Err(UnauthorizedAsSeller);
    }

    // Pools cannot be rescinded if the buyer has already accepted
    if buyer_has_accepted(&deps)? {
        return Err(IllegalRescindRequest);
    }

    let mut updated_seller_state = retrieve_seller_state(deps.storage)?;
    if updated_seller_state.pool_coins.is_empty() {
        return Err(InvalidRescindRequest);
    }

    let transfer_messages = create_send_coin_back_to_seller_messages(
        &deps,
        env.contract.address.to_string(),
        updated_seller_state.seller_address.to_string(),
    )?;

    let response = Response::new().add_messages(transfer_messages);

    // The contract no longer owns the denoms, so clear the list
    updated_seller_state.pool_coins = vec![];
    save_seller_state(deps.storage, &updated_seller_state)?;

    Ok(response.add_attribute("seller_state", format!("{:?}", updated_seller_state)))
}
