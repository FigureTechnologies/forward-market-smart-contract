use crate::error::ContractError;
use crate::error::ContractError::{InvalidDealerResetRequest, UnauthorizedDisableRequest};
use crate::storage::state_store::{clear_buyer_state, retrieve_contract_config, retrieve_optional_seller_state, save_contract_config, save_seller_state};
use crate::util::helpers::{create_send_coin_back_to_seller_messages, is_contract_admin, is_dealer, seller_has_finalized};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

pub fn execute_disable_contract(
    deps: DepsMut,
    env: Env,
    info: MessageInfo
) -> Result<Response, ContractError> {
    // In order to disable the contract you must be either the contract admin or a dealer
    if !is_contract_admin(&deps, &info)? && !is_dealer(&deps, &info)? {
        return Err(UnauthorizedDisableRequest);
    }

    let mut response: Response = Response::new();
    if seller_has_finalized(&deps)? {
        let mut seller = match retrieve_optional_seller_state(deps.storage)? {
            None => return Err(InvalidDealerResetRequest),
            Some(seller) => seller,
        };

        let transfer_messages = create_send_coin_back_to_seller_messages(
            &deps,
            env.contract.address.to_string(),
            seller.seller_address.to_string(),
            seller.pool_denoms,
        )?;

        if !transfer_messages.is_empty() {
            response = response.add_messages(transfer_messages);
        }

        // The contract no longer owns the denoms, so clear the list
        seller.pool_denoms = vec![];
        save_seller_state(deps.storage, &seller)?;
    }

    clear_buyer_state(deps.storage);

    // Contract is ok to disable, set the flag
    let mut updated_contract_config = retrieve_contract_config(deps.storage)?;
    updated_contract_config.is_disabled = true;
    save_contract_config(deps.storage, &updated_contract_config)?;

    Ok(response.add_attribute("contract_config", format!("{:?}", updated_contract_config)))
}
