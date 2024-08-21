use crate::error::ContractError;
use crate::error::ContractError::{
    IllegalConfirmationRequest, InvalidConfirmationRequest, MissingMarkerBaseAccount,
    UnauthorizedPrivateSeller,
};
use crate::storage::state_store::{
    retrieve_contract_config, retrieve_seller_state, retrieve_token_data_state,
    save_settlement_data_state, SettlementData,
};
use crate::util::helpers::{
    buyer_has_accepted, get_balance, get_marker, is_dealer, seller_has_finalized,
};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use provwasm_std::types::provenance::marker::v1::{
    MarkerQuerier, MsgDeleteAccessRequest, MsgTransferRequest,
};

pub fn execute_dealer_confirm(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    // Only a dealer can confirm so throw an error if the sender is not in the list of dealers
    if !is_dealer(&deps, &info)? {
        return Err(IllegalConfirmationRequest);
    }

    // The seller must have finalized and the buyer must have accepted before the transfer can take place
    if !seller_has_finalized(&deps)? || !buyer_has_accepted(&deps)? {
        return Err(InvalidConfirmationRequest);
    }

    let config = retrieve_contract_config(deps.storage)?;
    let seller_state = retrieve_seller_state(deps.storage)?;
    let token_data = retrieve_token_data_state(deps.storage)?;

    // A private contract should not allow a state where the accepted seller is not in the list of allowed
    // sellers, but before we transfer anything run a sanity check
    if config.use_private_sellers
        && !config
            .allowed_sellers
            .contains(&seller_state.seller_address)
    {
        return Err(UnauthorizedPrivateSeller);
    }

    // Create a response to add the messages to
    let mut response = Response::new();

    // Get the address of the marker that all the assets will be held by so that we can transfer
    // all the scopes to it
    let forward_market_marker = get_marker(
        token_data.token_denom.clone(),
        &MarkerQuerier::new(&deps.querier),
    )?;
    let forward_market_base_address = match forward_market_marker.base_account {
        None => {
            return Err(MissingMarkerBaseAccount {
                denom: token_data.token_denom.clone(),
            })
        }
        Some(base_account) => base_account.address,
    };

    // Iterate over the list of denoms so that we can update the value owner of the pool markers to be the
    // forward market marker
    for denom in seller_state.pool_denoms {
        let held_coin = get_balance(&deps, denom.clone())?;
        response = response.add_message(MsgTransferRequest {
            amount: Some(held_coin.coin),
            administrator: env.contract.address.to_string(),
            from_address: env.contract.address.to_string(),
            to_address: forward_market_base_address.clone(),
        });
    }

    save_settlement_data_state(
        deps.storage,
        &SettlementData {
            block_height: env.block.height,
            settling_dealer: info.sender,
        },
    )?;

    // Remove the contract from the access list
    response = response.add_message(MsgDeleteAccessRequest {
        denom: forward_market_marker.denom,
        administrator: env.contract.address.to_string(),
        removed_address: env.contract.address.to_string(),
    });

    Ok(response)
}
