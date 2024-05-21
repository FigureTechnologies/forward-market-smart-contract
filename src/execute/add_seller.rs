use crate::error::ContractError;
use crate::error::ContractError::{
    AcceptedValueExceedsMaxFaceValue, AcceptedValueLessThanMinFaceValue,
    AcceptedValueMustBePositive, InvalidTickSizeValueMatch, SellerAlreadyExists,
    UnauthorizedPrivateSeller,
};

use crate::storage::state_store::{
    retrieve_contract_config, retrieve_optional_seller_state, save_seller_state, Seller,
};
use crate::util::helpers::is_valid_tick_size;
use cosmwasm_std::{DepsMut, MessageInfo, Response, Uint128};

pub fn execute_add_seller(
    deps: DepsMut,
    info: MessageInfo,
    accepted_value_cents: Uint128,
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

    // Make sure the seller is proposing a value that is positive and non-zero
    if accepted_value_cents <= Uint128::new(0) {
        return Err(AcceptedValueMustBePositive);
    }

    // The seller shouldn't be able to propose a value lower than the min face value
    if accepted_value_cents < config.min_face_value_cents {
        return Err(AcceptedValueLessThanMinFaceValue);
    }

    // The seller shouldn't be able to propose a value higher than the max face value
    if accepted_value_cents > config.max_face_value_cents {
        return Err(AcceptedValueExceedsMaxFaceValue);
    }

    // The accepted value must be compatible with the tick size
    if !is_valid_tick_size(config.tick_size, accepted_value_cents) {
        return Err(InvalidTickSizeValueMatch);
    }

    // Store the seller information
    let seller_state = Seller {
        seller_address: info.sender.clone(),
        accepted_value_cents,
        pool_denoms: vec![],
        offer_hash,
    };
    save_seller_state(deps.storage, &seller_state)?;

    Ok(Response::new().add_attribute("seller_state", format!("{:?}", seller_state)))
}
