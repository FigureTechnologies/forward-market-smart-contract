use crate::error::ContractError;
use crate::msg::{BuyerResponse, ConfigResponse, GetContractStateResponse, SellerResponse, SettlementDataResponse, VersionInfoResponse};
use crate::storage::state_store::{retrieve_buyer_state, retrieve_contract_config, retrieve_optional_seller_state, retrieve_optional_settlement_data_state, Seller, SettlementData};
use crate::version_info::get_version_info;
use cosmwasm_std::Deps;

pub fn query_contract_state(deps: Deps) -> Result<GetContractStateResponse, ContractError> {
    let buyer = retrieve_buyer_state(deps.storage)?;
    let seller = retrieve_optional_seller_state(deps.storage)?;
    let seller_response: Option<SellerResponse> = match seller {
        None => {None}
        Some(seller_state) => {
            Some(SellerResponse {
                seller_address: seller_state.seller_address,
                accepted_value_cents: seller_state.accepted_value_cents,
                pool_denoms: seller_state.pool_coins.into_iter().map(|coin| -> String {
                    coin.denom
                }).collect(),
                offer_hash: seller_state.offer_hash,
            })
        }
    };

    let settlement_data = retrieve_optional_settlement_data_state(deps.storage)?;
    let settlement_data_response: Option<SettlementDataResponse> = match settlement_data {
        None => {None}
        Some(state) => {
            Some(SettlementDataResponse {
                block_height: state.block_height,
                settling_dealer: state.settling_dealer,
            })
        }
    };
    let config = retrieve_contract_config(deps.storage)?;

    let version_info = get_version_info(deps.storage)?;
    let response = GetContractStateResponse {
        buyer: BuyerResponse {
            buyer_address: buyer.buyer_address,
            has_accepted_pools: buyer.has_accepted_pools,
        },
        seller: seller_response,
        config: ConfigResponse {
            is_private: config.is_private,
            allowed_sellers: config.allowed_sellers,
            agreement_terms_hash: config.agreement_terms_hash,
            token_denom: config.token_denom,
            max_face_value_cents: config.max_face_value_cents,
            min_face_value_cents: config.min_face_value_cents,
            tick_size: config.tick_size,
            dealers: config.dealers,
            is_disabled: config.is_disabled,
        },
        settlement_data: settlement_data_response,
        version_info: VersionInfoResponse {
            version: version_info.version,
            definition: version_info.definition
        },
    };
    Ok(response)
}
