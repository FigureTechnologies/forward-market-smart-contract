use crate::error::ContractError;
use crate::error::ContractError::{
    IllegalCoinOwnership, InvalidFinalizationRequest, PoolAlreadyFinalized, UnauthorizedAsSeller,
};
use crate::storage::state_store::{retrieve_seller_state, save_seller_state};
use crate::util::helpers::{get_balance, is_seller, seller_has_finalized};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use provwasm_std::types::provenance::marker::v1::{MsgWithdrawRequest};

pub fn execute_finalize_pools(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    // Only the seller can finalize the seller pool denom list
    if !is_seller(&deps, &info)? {
        return Err(UnauthorizedAsSeller);
    }

    // In order for the transaction to take place, we need to have at least one pool
    if info.funds.is_empty() {
        return Err(InvalidFinalizationRequest);
    }

    // Return an error if the seller has already finalized
    if seller_has_finalized(&deps)? {
        return Err(PoolAlreadyFinalized);
    }

    let mut response = Response::new();

    // Set the state to show the seller has finalized
    let mut updated_seller = retrieve_seller_state(deps.storage)?;
    updated_seller.pool_coins = info.funds.clone();
    save_seller_state(deps.storage, &updated_seller)?;
    Ok(response.add_attribute("seller_state", format!("{:?}", updated_seller)))
}
