use crate::error::ContractError;
use crate::error::ContractError::{
    AcceptedValueExceedsMaxFaceValue, AcceptedValueLessThanMinFaceValue,
    AcceptedValueMustBePositive, InvalidAgreementTermsHash, InvalidTickSizeValueMatch,
    SellerAlreadyExists, UnauthorizedPrivateSeller,
};

use crate::storage::state_store::{
    retrieve_buyer_state, retrieve_contract_config, retrieve_optional_seller_state,
    save_seller_state, Seller,
};
use crate::util::helpers::{create_and_transfer_marker, is_valid_tick_size};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, Uint128};
use std::ops::Div;

pub fn execute_add_seller(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    accepted_value_cents: Uint128,
    offer_hash: String,
    agreement_terms_hash: String,
) -> Result<Response, ContractError> {
    // Make sure we haven't already set the seller config. If we have, return an error
    match retrieve_optional_seller_state(deps.storage)? {
        None => {}
        Some(_) => return Err(SellerAlreadyExists),
    }

    let config = retrieve_contract_config(deps.storage)?;

    // If the contract is private, make sure this seller is allowed to engage with it
    if config.is_private && !config.allowed_sellers.contains(&info.sender) {
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

    // The seller passes the agreement terms hash for the terms they are agreeing to. We check
    // the hash against what is stored in the contract to make sure the seller isn't agreeing to
    // a previous state of the contract
    if config.agreement_terms_hash != agreement_terms_hash {
        return Err(InvalidAgreementTermsHash);
    }

    // Store the seller information
    let seller_state = Seller {
        seller_address: info.sender.clone(),
        accepted_value_cents,
        pool_denoms: vec![],
        offer_hash,
    };
    save_seller_state(deps.storage, &seller_state)?;

    let number_of_coins = accepted_value_cents.clone().div(config.tick_size.clone());

    let buyer_state = retrieve_buyer_state(deps.storage)?;

    // Now that we have a seller, we can create the forward market token and give it to the buyer
    let create_token_messages = create_and_transfer_marker(
        env.contract.address.to_string(),
        config.token_denom,
        number_of_coins,
        buyer_state.buyer_address.to_string(),
        config.dealers.clone(),
    );

    Ok(Response::new()
        .add_messages(create_token_messages)
        .add_attribute("seller_state", format!("{:?}", seller_state)))
}
