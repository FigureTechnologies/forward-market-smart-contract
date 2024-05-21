use crate::error::ContractError;
use crate::error::ContractError::{
    IllegalAcceptingParty, IllegalPoolAcceptanceRequest, PoolAlreadyAccepted,
};
use crate::storage::state_store::{save_transaction_state, TransactionState};
use crate::util::helpers::{buyer_has_accepted, is_buyer, seller_has_finalized};
use cosmwasm_std::{DepsMut, MessageInfo, Response};

pub fn execute_accept_finalized_pools(
    deps: DepsMut,
    info: MessageInfo,
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

    let transaction_state = TransactionState {
        buyer_address: info.sender,
        buyer_has_accepted_pools: true,
    };

    // Update the transaction state to reflect that the buyer has accepted
    save_transaction_state(deps.storage, &transaction_state)?;

    Ok(Response::new().add_attribute("transaction_state", format!("{:?}", transaction_state)))
}
