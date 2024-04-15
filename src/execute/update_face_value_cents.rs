use crate::error::ContractError;
use crate::storage::state_store::retrieve_contract_config;
use crate::util::helpers::{update_config_as_buyer, validate_face_values};
use cosmwasm_std::{DepsMut, MessageInfo, Response, Uint128};

pub fn execute_update_face_value_cents(
    deps: DepsMut,
    info: MessageInfo,
    new_min_face_value_cents: Uint128,
    new_max_face_value_cents: Uint128,
    new_tick_size: Uint128,
) -> Result<Response, ContractError> {
    // Validate the tick size against the min and max values
    validate_face_values(
        new_min_face_value_cents,
        new_max_face_value_cents,
        new_tick_size,
    )?;

    let mut updated_config = retrieve_contract_config(deps.storage)?.clone();
    updated_config.min_face_value_cents = new_min_face_value_cents;
    updated_config.max_face_value_cents = new_max_face_value_cents;
    updated_config.tick_size = new_tick_size;
    update_config_as_buyer(deps, info, updated_config)
}
