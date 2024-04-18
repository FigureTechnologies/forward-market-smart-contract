use crate::error::ContractError;
use crate::error::ContractError::{
    IllegalCoinOwnership, InvalidFinalizationRequest, PoolAlreadyFinalized, UnauthorizedAsSeller,
};
use crate::storage::state_store::{retrieve_seller_state, save_seller_state};
use crate::util::helpers::{get_balance, is_seller, seller_has_finalized};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use provwasm_std::types::provenance::marker::v1::MsgTransferRequest;

pub fn execute_finalize_pools(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    pool_denoms: &Vec<String>,
) -> Result<Response, ContractError> {
    // Only the seller can finalize the seller pool denom list
    if !is_seller(&deps, &info)? {
        return Err(UnauthorizedAsSeller);
    }

    // In order for the transaction to take place, we need to have at least one pool
    if pool_denoms.is_empty() {
        return Err(InvalidFinalizationRequest);
    }

    // Return an error if the seller has already finalized
    if seller_has_finalized(&deps)? {
        return Err(PoolAlreadyFinalized);
    }

    let mut response = Response::new();
    // Iterate over the list of denoms so that we can transfer the coin to the contract
    for denom in pool_denoms {
        let held_coin = get_balance(&deps, denom.clone())?;

        // The seller must own the coins that are being added to the contract
        if held_coin.address != info.sender.to_string() {
            return Err(IllegalCoinOwnership);
        }

        // Transfer the coin to the contract
        response = response.add_message(MsgTransferRequest {
            amount: Some(held_coin.coin),
            administrator: env.contract.address.to_string(),
            from_address: held_coin.address,
            to_address: env.contract.address.to_string(),
        });
    }

    // Set the state to show the seller has finalized
    let mut updated_seller = retrieve_seller_state(deps.storage)?;
    updated_seller.pool_denoms = pool_denoms.clone();
    save_seller_state(deps.storage, &updated_seller)?;
    Ok(response.add_attribute("seller_state", format!("{:?}", updated_seller)))
}
