use crate::error::ContractError;
use crate::error::ContractError::{MaxPrivateBuyersReached, UnauthorizedPrivateBuyer};
use crate::storage::state_store::{
    retrieve_bid_list_state, retrieve_contract_config, save_bid_list_state, Bid, BidList,
};
use cosmwasm_std::{DepsMut, MessageInfo, Response};

pub fn execute_add_bidder(
    deps: DepsMut,
    info: MessageInfo,
    agreement_terms_hash: String,
) -> Result<Response, ContractError> {
    let config = retrieve_contract_config(deps.storage)?;
    let bid_list = retrieve_bid_list_state(deps.storage)?;

    // If using private buyers, make sure this buyer is allowed to submit a bid
    if config.use_private_buyers {
        if !config.allowed_buyers.contains(&info.sender) {
            return Err(UnauthorizedPrivateBuyer);
        }

        if bid_list.bids.len() >= usize::try_from(config.max_bid_count).unwrap() {
            return Err(MaxPrivateBuyersReached);
        }
    }

    // Remove any existing bid for this buyer because if one exists we want to replace it
    let mut updated_bid_list: Vec<Bid> = bid_list
        .bids
        .into_iter()
        .filter(|bid| bid.buyer_address != info.sender)
        .collect();
    updated_bid_list.push(Bid {
        buyer_address: info.sender,
        agreement_terms_hash,
    });

    // Save the updated buyer state
    save_bid_list_state(
        deps.storage,
        &BidList {
            bids: updated_bid_list.clone(),
        },
    )?;

    Ok(Response::new().add_attribute("bid_list", format!("{:?}", updated_bid_list.clone())))
}
