use crate::error::ContractError;
use crate::error::ContractError::{
    BidDoesNotExist, BidPreviouslyAccepted, InvalidAgreementTermsHash, TokensNotMinted,
    UnauthorizedAsSeller,
};
use crate::storage::state_store::{
    retrieve_bid_list_state, retrieve_optional_buyer_state, retrieve_optional_token_data_state,
    save_buyer_state, Bid, Buyer,
};
use crate::util::helpers::{create_transfer_tokens_message, is_seller};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

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

    // Tokens must be minted before bid acceptance because the FM token is sent to the buyer
    // upon bid acceptance
    let token_data = match retrieve_optional_token_data_state(deps.storage)? {
        None => return Err(TokensNotMinted),
        Some(token_data) => token_data,
    };

    let bidder_address = deps.api.addr_validate(&bidder_address_str)?;

    // Make sure the bidder address exists in the list of bids
    let bid_list = retrieve_bid_list_state(deps.storage)?;
    let bid: Option<Bid> = bid_list.bids.into_iter().find(|existing_bid| -> bool {
        existing_bid.buyer_address.to_string() == bidder_address_str
    });

    match bid {
        None => {
            return Err(BidDoesNotExist {
                address: bidder_address.to_string(),
            });
        }
        // Check the provided hash against the hash in the state to make sure the seller is
        // accepting the terms the agree to
        Some(bid_state) => {
            if bid_state.agreement_terms_hash != agreement_terms_hash {
                return Err(InvalidAgreementTermsHash);
            }
        }
    }

    // Return an error if the seller has already accepted a bid
    match retrieve_optional_buyer_state(deps.storage)? {
        None => {}
        Some(buyer) => {
            return Err(BidPreviouslyAccepted {
                address: buyer.buyer_address.to_string(),
            })
        }
    }

    let buyer = Buyer {
        buyer_address: bidder_address.clone(),
        buyer_has_accepted_pools: false,
        agreement_terms_hash: agreement_terms_hash.clone(),
    };
    save_buyer_state(deps.storage, &buyer)?;

    // Now that we have a buyer, we can give them the forward market tokens
    let create_token_messages = create_transfer_tokens_message(
        env.contract.address.to_string(),
        token_data.token_denom,
        token_data.token_count,
        bidder_address.to_string(),
    );

    Ok(Response::new()
        .add_messages(create_token_messages)
        .add_attribute("buyer", format!("{:?}", buyer)))
}
