use crate::error::ContractError;
use crate::error::ContractError::{IllegalDealerResetRequest, InvalidDealerResetRequest};
use crate::storage::state_store::{
    clear_buyer_state, retrieve_optional_seller_state, retrieve_optional_buyer_state,
    save_seller_state,
};
use crate::util::helpers::{create_send_coin_back_to_seller_messages, is_dealer};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

pub fn execute_dealer_reset(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    if !is_dealer(&deps, &info)? {
        return Err(IllegalDealerResetRequest);
    }

    let mut updated_seller = match retrieve_optional_seller_state(deps.storage)? {
        None => return Err(InvalidDealerResetRequest),
        Some(seller) => seller,
    };

    let transfer_messages = create_send_coin_back_to_seller_messages(
        &deps,
        env.contract.address.to_string(),
        updated_seller.seller_address.to_string(),
        updated_seller.pool_denoms,
    )?;

    let mut response = Response::new();
    if !transfer_messages.is_empty() {
        response = response.add_messages(transfer_messages);
    }

    // The contract no longer owns the denoms, so clear the list
    updated_seller.pool_denoms = vec![];
    save_seller_state(deps.storage, &updated_seller)?;

    clear_buyer_state(deps.storage);

    match retrieve_optional_buyer_state(deps.storage)? {
        None => {}
        Some(_) => {}
    }

    Ok(response.add_attribute("seller_state", format!("{:?}", updated_seller)))
}
