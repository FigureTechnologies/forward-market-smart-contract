use crate::error::ContractError;
use crate::error::ContractError::{
    BuyerAlreadyExists, BuyerDoesNotExist, InvalidAgreementTermsHash, UnauthorizedAsSeller,
};
use crate::storage::state_store::{
    retrieve_buyer_state, retrieve_contract_config, retrieve_optional_transaction_state,
    retrieve_seller_state, save_transaction_state, Buyer, TransactionState,
};
use crate::util::helpers::{create_and_transfer_marker, is_seller};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use std::ops::Div;

pub fn execute_accept_buyer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    buyer_address_str: String,
    agreement_terms_hash: String,
) -> Result<Response, ContractError> {
    // Only the seller can accept a buyer's bid
    if !is_seller(&deps, &info)? {
        return Err(UnauthorizedAsSeller);
    }

    let buyer_address = deps.api.addr_validate(&buyer_address_str)?;

    // Make sure the buyer address exists in the list of buyer bids
    let buyer_state = retrieve_buyer_state(deps.storage)?;
    let buyer: Option<Buyer> = buyer_state
        .buyers
        .into_iter()
        .find(|buyer| -> bool { buyer.buyer_address == buyer_address_str });

    match buyer {
        None => {
            return Err(BuyerDoesNotExist {
                address: buyer_address.to_string(),
            });
        }
        Some(buyer_state) => {
            if buyer_state.agreement_terms_hash != agreement_terms_hash {
                return Err(InvalidAgreementTermsHash);
            }
        }
    }

    // Return an error if the seller has already accepted a bid
    match retrieve_optional_transaction_state(deps.storage)? {
        None => {}
        Some(transaction_state) => {
            return Err(BuyerAlreadyExists {
                address: transaction_state.buyer_address.to_string(),
            })
        }
    }

    let transaction_state = TransactionState {
        buyer_address: buyer_address.clone(),
        buyer_has_accepted_pools: false,
        agreement_terms_hash
    };
    save_transaction_state(deps.storage, &transaction_state)?;

    let seller_state = retrieve_seller_state(deps.storage)?;
    let config = retrieve_contract_config(deps.storage)?;

    let number_of_coins = seller_state
        .accepted_value_cents
        .clone()
        .div(config.tick_size.clone());

    // Now that we have a buyer, we can create the forward market token and give it to the buyer
    let create_token_messages = create_and_transfer_marker(
        env.contract.address.to_string(),
        config.token_denom,
        number_of_coins,
        buyer_address.to_string(),
        config.dealers.clone(),
    );

    Ok(Response::new()
        .add_messages(create_token_messages)
        .add_attribute("transaction_state", format!("{:?}", transaction_state)))
}
