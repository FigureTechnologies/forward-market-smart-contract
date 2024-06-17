use crate::error::ContractError;
use crate::error::ContractError::{
    BidPreviouslyAccepted, BidDoesNotExist, InvalidAgreementTermsHash, UnauthorizedAsSeller,
};
use crate::storage::state_store::{
    retrieve_bid_list_state, retrieve_contract_config, retrieve_optional_transaction_state,
    retrieve_seller_state, save_transaction_state, Bid, TransactionState,
};
use crate::util::helpers::{create_and_transfer_marker, is_seller};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use std::ops::Div;

pub fn execute_accept_bid(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    bidder_address_str: String,
    agreement_terms_hash: String,
) -> Result<Response, ContractError> {
    // Only the seller can accept a buyer's bid
    if !is_seller(&deps, &info)? {
        return Err(UnauthorizedAsSeller);
    }

    let bidder_address = deps.api.addr_validate(&bidder_address_str)?;

    // Make sure the bidder address exists in the list of bids
    let bid_list = retrieve_bid_list_state(deps.storage)?;
    let bid: Option<Bid> = bid_list
        .bids
        .into_iter()
        .find(|existing_bid| -> bool { existing_bid.buyer_address == bidder_address_str });

    match bid {
        None => {
            return Err(BidDoesNotExist {
                address: bidder_address.to_string(),
            });
        }
        Some(bid_state) => {
            if bid_state.agreement_terms_hash != agreement_terms_hash {
                return Err(InvalidAgreementTermsHash);
            }
        }
    }

    // Return an error if the seller has already accepted a bid
    match retrieve_optional_transaction_state(deps.storage)? {
        None => {}
        Some(transaction_state) => {
            return Err(BidPreviouslyAccepted {
                address: transaction_state.buyer_address.to_string(),
            })
        }
    }

    let transaction_state = TransactionState {
        buyer_address: bidder_address.clone(),
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
        bidder_address.to_string(),
        config.dealers.clone(),
    );

    Ok(Response::new()
        .add_messages(create_token_messages)
        .add_attribute("transaction_state", format!("{:?}", transaction_state)))
}
