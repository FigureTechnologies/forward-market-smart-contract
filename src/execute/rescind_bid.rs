use cosmwasm_std::{DepsMut, MessageInfo, Response};
use crate::error::ContractError;
use crate::error::ContractError::{IllegalBidRescind, InvalidBidRescind};
use crate::storage::state_store::{Bid, BidList, retrieve_bid_list_state, save_bid_list_state};
use crate::util::helpers::is_buyer;

pub fn execute_rescind_bid(
    deps: DepsMut,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    // Bid cannot be rescinded if the bid has already been accepted by the seller
    if is_buyer(&deps, &info)? {
        return Err(IllegalBidRescind)
    }
    let bid_list = retrieve_bid_list_state(deps.storage)?.bids;

    // Get the number of current bids
    let bid_count = bid_list.len();

    // Remove the bid that corresponds to the sender's address
    let updated_bid_list: Vec<Bid> = bid_list
        .into_iter()
        .filter(|bid| bid.buyer_address != info.sender)
        .collect();

    // If the bid count is unchanged it means we didn't find a bid to remove, so return an error
    if updated_bid_list.len() == bid_count {
        return Err(InvalidBidRescind)
    }

    // Save the updated buyer state
    save_bid_list_state(
        deps.storage,
        &BidList {
            bids: updated_bid_list.clone(),
        },
    )?;

    Ok(Response::new().add_attribute("bid_list", format!("{:?}", updated_bid_list.clone())))
}
