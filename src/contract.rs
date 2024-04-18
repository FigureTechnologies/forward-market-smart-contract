use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response,
};

use crate::error::ContractError;
use crate::error::ContractError::{IllegalContractExecution, InvalidContractExecution};
use crate::execute::accept_finalized_pools::execute_accept_finalized_pools;
use crate::execute::add_seller::execute_add_seller;
use crate::execute::dealer_confirm::execute_dealer_confirm;
use crate::execute::dealer_reset::execute_dealer_reset;
use crate::execute::disable_contract::execute_disable_contract;
use crate::execute::finalize_pools::execute_finalize_pools;
use crate::execute::remove_as_seller::execute_remove_as_seller;
use crate::execute::rescind_finalized_pools::execute_rescind_finalized_pools;
use crate::execute::update_allowed_sellers::execute_update_allowed_sellers;
use crate::execute::update_face_value_cents::execute_update_face_value_cents;
use crate::execute::update_terms_hash::execute_update_terms_hash;
use crate::instantiate::instantiate_contract::instantiate_contract;
use crate::msg::{ExecuteMsg, InstantiateContractMsg, QueryMsg};
use crate::query::contract_state::query_contract_state;
use crate::storage::state_store::{
    retrieve_contract_config, retrieve_optional_settlement_data_state,
};

/// The entry point used when an account instantiates a stored code wasm payload of this contract on
/// the Provenance Blockchain.
///
/// # Parameters
///
/// * `deps` A dependencies object provided by the cosmwasm framework.  Allows access to useful
/// resources like contract internal storage and a querier to retrieve blockchain objects.
/// * `env` An environment object provided by the cosmwasm framework.  Describes the contract's
/// details, as well as blockchain information at the time of the transaction.
/// * `info` A message information object provided by the cosmwasm framework.  Describes the sender
/// of the instantiation message, as well as the funds provided as an amount during the transaction.
/// * `msg` A custom instantiation message defined by this contract for creating the initial
/// configuration used by the contract.
#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateContractMsg,
) -> Result<Response, ContractError> {
    instantiate_contract(deps, env, info, msg)
}

/// The entry point used when an account initiates an execution process defined in the contract.
/// This defines the primary operations of the contract.
///
/// # Parameters
///
/// * `deps` A dependencies object provided by the cosmwasm framework.  Allows access to useful
/// resources like contract internal storage and a querier to retrieve blockchain objects.
/// * `env` An environment object provided by the cosmwasm framework.  Describes the contract's
/// details, as well as blockchain information at the time of the transaction.
/// * `info` A message information object provided by the cosmwasm framework.  Describes the sender
/// of the instantiation message, as well as the funds provided as an amount during the transaction.
/// * `msg` A custom execution message enum defined by this contract that will map the desired operation
/// to the proper contract logic.
#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    // If the contract has already been settled, no further execution
    let has_settled = retrieve_optional_settlement_data_state(deps.storage)?.is_some();
    if has_settled {
        return Err(IllegalContractExecution);
    }

    // If the contract has been disabled, no further execution is allowed
    let is_disabled = retrieve_contract_config(deps.storage)?.is_disabled;
    if is_disabled {
        return Err(InvalidContractExecution);
    }
    match msg {
        ExecuteMsg::AddSeller {
            accepted_value_cents,
            offer_hash,
            agreement_terms_hash,
        } => execute_add_seller(
            deps,
            env,
            info,
            accepted_value_cents,
            offer_hash,
            agreement_terms_hash,
        ),
        ExecuteMsg::RemoveAsSeller {} => execute_remove_as_seller(deps, info),
        ExecuteMsg::FinalizePools {} => execute_finalize_pools(deps, info),
        ExecuteMsg::DealerConfirm {} => execute_dealer_confirm(deps, env, info),
        ExecuteMsg::UpdateAgreementTermsHash {
            agreement_terms_hash: new_agreement_terms_hash,
        } => execute_update_terms_hash(deps, info, new_agreement_terms_hash),
        ExecuteMsg::UpdateFaceValueCents {
            max_face_value_cents,
            min_face_value_cents,
            tick_size,
        } => execute_update_face_value_cents(
            deps,
            info,
            min_face_value_cents,
            max_face_value_cents,
            tick_size,
        ),
        ExecuteMsg::UpdateAllowedSellers { allowed_sellers } => {
            execute_update_allowed_sellers(deps, info, allowed_sellers)
        }
        ExecuteMsg::AcceptFinalizedPools {} => execute_accept_finalized_pools(deps, info),
        ExecuteMsg::RescindFinalizedPools {} => execute_rescind_finalized_pools(deps, env, info),
        ExecuteMsg::DealerReset {} => execute_dealer_reset(deps, env, info),
        ExecuteMsg::ContractDisable {} => execute_disable_contract(deps, info),
    }
}

/// The entry point used when an account invokes the contract to retrieve information.  Allows
/// read-only access to the contract state.
///
/// # Parameters
///
/// * `deps` A dependencies object provided by the cosmwasm framework.  Allows access to useful
/// resources like contract internal storage and a querier to retrieve blockchain objects.
/// * `_env` An environment object provided by the cosmwasm framework.  Describes the contract's
/// details, as well as blockchain information at the time of the transaction.  Unused by this
/// function, but required by cosmwasm for successfully defined query entrypoint.
/// * `msg` A custom query message enum defined by this contract that will map the desired query
/// to the proper contract logic
#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::GetContractState {} => Ok(to_json_binary(&query_contract_state(deps)?)?),
    }
}
