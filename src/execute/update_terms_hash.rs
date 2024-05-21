use crate::error::ContractError;
use crate::storage::state_store::{retrieve_buyer_state, retrieve_contract_config};
use crate::util::helpers::update_config_as_admin;
use cosmwasm_std::{DepsMut, MessageInfo, Response};

pub fn execute_update_bid(
    deps: DepsMut,
    info: MessageInfo,
    new_terms_hash: String,
) -> Result<Response, ContractError> {
    let buyer_state = retrieve_buyer_state(deps.storage)?;

    let mut updated_config = retrieve_contract_config(deps.storage)?.clone();
    updated_config.agreement_terms_hash = new_terms_hash;
    update_config_as_admin(deps, info, updated_config)
}
