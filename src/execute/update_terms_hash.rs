use crate::error::ContractError;
use crate::storage::state_store::retrieve_contract_config;
use crate::util::helpers::update_config_as_buyer;
use cosmwasm_std::{DepsMut, MessageInfo, Response};

pub fn execute_update_terms_hash(
    deps: DepsMut,
    info: MessageInfo,
    new_terms_hash: String,
) -> Result<Response, ContractError> {
    let mut updated_config = retrieve_contract_config(deps.storage)?.clone();
    updated_config.agreement_terms_hash = new_terms_hash;
    update_config_as_buyer(deps, info, updated_config)
}
