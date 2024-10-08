use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response,
};

use crate::error::ContractError;
use crate::error::ContractError::{IllegalContractExecution, InvalidContractExecution};
use crate::execute::accept_bid::execute_accept_bid;
use crate::execute::accept_finalized_pools::execute_accept_finalized_pools;
use crate::execute::add_bidder::execute_add_bidder;
use crate::execute::add_seller::execute_add_seller;
use crate::execute::dealer_confirm::execute_dealer_confirm;
use crate::execute::disable_contract::execute_disable_contract;
use crate::execute::finalize_pools::execute_finalize_pools;
use crate::execute::mint_tokens::execute_mint_tokens;
use crate::execute::rescind_finalized_pools::execute_rescind_finalized_pools;
use crate::execute::update_allowed_buyers::execute_update_allowed_buyers;
use crate::execute::update_allowed_sellers::execute_update_allowed_sellers;
use crate::execute::update_seller_offer_hash::execute_update_seller_offer_hash;
use crate::instantiate::instantiate_contract::instantiate_contract;
use crate::migrate::migrate::migrate_contract;
use crate::msg::{ExecuteMsg, InstantiateContractMsg, MigrateMsg, QueryMsg};
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
        ExecuteMsg::AddSeller { offer_hash } => execute_add_seller(deps, info, offer_hash),
        ExecuteMsg::UpdateSellerOfferHash { offer_hash } => {
            execute_update_seller_offer_hash(deps, info, offer_hash)
        }
        ExecuteMsg::FinalizePools { pool_denoms } => {
            execute_finalize_pools(deps, env, info, &pool_denoms)
        }
        ExecuteMsg::DealerConfirm {} => execute_dealer_confirm(deps, env, info),
        ExecuteMsg::UpdateAllowedSellers { allowed_sellers } => {
            execute_update_allowed_sellers(deps, info, allowed_sellers)
        }
        ExecuteMsg::UpdateAllowedBuyers { allowed_buyers } => {
            execute_update_allowed_buyers(deps, info, allowed_buyers)
        }
        ExecuteMsg::AcceptFinalizedPools { offer_hash } => {
            execute_accept_finalized_pools(deps, info, offer_hash)
        }
        ExecuteMsg::RescindFinalizedPools {} => execute_rescind_finalized_pools(deps, env, info),
        ExecuteMsg::ContractDisable {} => execute_disable_contract(deps, env, info),
        ExecuteMsg::AcceptBid {
            bidder_address,
            agreement_terms_hash,
        } => execute_accept_bid(deps, env, info, bidder_address, agreement_terms_hash),
        ExecuteMsg::AddBid {
            agreement_terms_hash,
        } => execute_add_bidder(deps, info, agreement_terms_hash),
        ExecuteMsg::MintTokens {
            token_count,
            token_denom,
        } => execute_mint_tokens(deps, env, info, token_count, token_denom),
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

/// The entry point used when the contract admin migrates an existing instance of this contract to
/// a new stored code instance on chain.
///
/// # Parameters
///
/// * `deps` A dependencies object provided by the cosmwasm framework.  Allows access to useful
/// resources like contract internal storage and a querier to retrieve blockchain objects.
/// * `env` An environment object provided by the cosmwasm framework.  Describes the contract's
/// details, as well as blockchain information at the time of the transaction.
/// * `msg` A custom migration message defined by this contract that will map the desired operation
/// to the proper contract logic.
#[entry_point]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    match msg {
        MigrateMsg::ContractUpgrade {} => migrate_contract(deps),
    }
}
