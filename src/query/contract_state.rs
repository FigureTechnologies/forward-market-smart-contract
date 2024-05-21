use crate::error::ContractError;
use crate::msg::GetContractStateResponse;
use crate::storage::state_store::{
    retrieve_buyer_state, retrieve_contract_config, retrieve_optional_seller_state,
    retrieve_optional_settlement_data_state, retrieve_optional_transaction_state,
};
use crate::version_info::get_version_info;
use cosmwasm_std::Deps;

pub fn query_contract_state(deps: Deps) -> Result<GetContractStateResponse, ContractError> {
    let buyers = retrieve_buyer_state(deps.storage)?;
    let seller = retrieve_optional_seller_state(deps.storage)?;
    let config = retrieve_contract_config(deps.storage)?;
    let settlement_data = retrieve_optional_settlement_data_state(deps.storage)?;
    let version_info = get_version_info(deps.storage)?;
    let transaction_state = retrieve_optional_transaction_state(deps.storage)?;
    let response = GetContractStateResponse {
        buyers,
        seller,
        config,
        settlement_data,
        version_info,
        transaction_state,
    };
    Ok(response)
}
