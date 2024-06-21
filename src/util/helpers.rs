use crate::error::ContractError;
use crate::error::ContractError::{
    IllegalConfigUpdate, InvalidDenom, InvalidDenomOwnership, UnauthorizedConfigUpdate,
};
use crate::msg::KeyType::Session;
use crate::msg::{KeyType, MetadataAddress};
use crate::storage::state_store::{
    retrieve_contract_config, retrieve_optional_seller_state, retrieve_optional_buyer_state,
    save_contract_config, Config,
};
use bech32::ToBase32;
use cosmwasm_std::{
    Addr, CosmosMsg, DepsMut, Empty, MessageInfo, QuerierWrapper, Response, StdError, StdResult,
    Uint128,
};
use provwasm_std::types::cosmos::base::query::v1beta1::PageRequest;
use provwasm_std::types::cosmos::base::v1beta1::Coin;
use provwasm_std::types::provenance::marker::v1::{
    Access, AccessGrant, MarkerAccount, MarkerQuerier, MarkerStatus, MarkerType,
    MsgActivateRequest, MsgAddMarkerRequest, MsgFinalizeRequest, MsgTransferRequest,
    MsgWithdrawRequest, QueryHoldingRequest, QueryHoldingResponse,
};
use provwasm_std::types::provenance::metadata::v1::{MetadataQuerier, ValueOwnershipResponse};
use uuid::Uuid;

pub fn create_and_transfer_marker(
    contract_address: String,
    denom: String,
    amount: Uint128,
    owner_address: String,
    dealer_list: Vec<Addr>,
) -> Vec<CosmosMsg> {
    let mut messages: Vec<CosmosMsg> = vec![];
    let coin = Coin {
        denom: denom.clone(),
        amount: amount.to_string(),
    };

    let mut access_grants = vec![];
    for access_address in dealer_list {
        // Give the dealers access to withdraw and transfer only
        access_grants.push(AccessGrant {
            address: access_address.to_string(),
            permissions: vec![Access::Withdraw as i32, Access::Deposit as i32],
        })
    }

    // The contract will have full access to the marker but contract's access is removed
    // when the transaction is settled
    access_grants.push(AccessGrant {
        address: contract_address.clone(),
        permissions: vec![
            Access::Admin as i32,
            Access::Burn as i32,
            Access::Mint as i32,
            Access::Deposit as i32,
            Access::Withdraw as i32,
            Access::Delete as i32,
        ],
    });

    messages.push(CosmosMsg::from(MsgAddMarkerRequest {
        amount: Some(coin),
        manager: contract_address.clone(),
        from_address: contract_address.to_string(),
        status: MarkerStatus::Proposed as i32,
        marker_type: MarkerType::Coin as i32,
        access_list: access_grants,
        supply_fixed: false,
        allow_governance_control: true,
        allow_forced_transfer: false,
        required_attributes: vec![],
    }));

    messages.push(CosmosMsg::from(MsgFinalizeRequest {
        denom: denom.clone(),
        administrator: contract_address.clone(),
    }));

    messages.push(CosmosMsg::from(MsgActivateRequest {
        denom: denom.clone(),
        administrator: contract_address.clone(),
    }));

    messages.push(CosmosMsg::from(MsgWithdrawRequest {
        denom: denom.to_string(),
        administrator: contract_address.clone(),
        to_address: owner_address.clone(),
        amount: vec![Coin {
            denom: denom.clone(),
            amount: amount.to_string(),
        }],
    }));
    messages
}

pub fn get_owned_scopes(
    marker_address: String,
    querier: &QuerierWrapper,
    offset: u64,
    limit: u64,
) -> Result<ValueOwnershipResponse, StdError> {
    let metadata_querier = MetadataQuerier::new(querier);
    metadata_querier.value_ownership(
        marker_address,
        Some(PageRequest {
            key: vec![],
            offset,
            limit,
            count_total: false,
            reverse: false,
        }),
    )
}

pub fn get_marker(id: String, querier: &MarkerQuerier<Empty>) -> StdResult<MarkerAccount> {
    let response = querier.marker(id)?;
    if let Some(marker) = response.marker {
        return if let Ok(account) = MarkerAccount::try_from(marker) {
            Ok(account)
        } else {
            Err(StdError::generic_err("unable to type-cast marker account"))
        };
    } else {
        Err(StdError::generic_err("no marker found for id"))
    }
}

pub fn encode_bech32(key_type: KeyType, bytes: &Vec<u8>) -> Result<String, bech32::Error> {
    bech32::encode(
        key_type.to_str(),
        bytes.to_base32(),
        bech32::Variant::Bech32,
    )
}

pub fn scope(scope_uuid: Uuid) -> Result<MetadataAddress, ContractError> {
    let key_type_byte = KeyType::Scope as u8;
    let bytes = [key_type_byte]
        .iter()
        .cloned()
        .chain(hex::decode(scope_uuid.simple().encode_lower(&mut Uuid::encode_buffer())).unwrap())
        .collect::<Vec<u8>>();
    let addr = encode_bech32(Session, &bytes).unwrap();
    Ok(MetadataAddress {
        bech32: addr,
        bytes,
        key_type: KeyType::Scope,
    })
}

pub fn get_balance(deps: &DepsMut, denom: String) -> Result<HeldCoin, ContractError> {
    // Partial ownership of assets being contributed is not supported, so we check to make sure that the
    // coins are only held by one address
    let holding_response: QueryHoldingResponse = deps.querier.query(
        &QueryHoldingRequest {
            id: denom.clone(),
            pagination: None,
        }
        .into(),
    )?;
    if holding_response.balances.len() > 1 {
        return Err(InvalidDenomOwnership {
            denom: denom.clone(),
        });
    }

    // Querying by denom should return a single balance and a single coin, if this is not the
    // case we are in an error state
    if holding_response.balances.len() != 1 {
        return Err(InvalidDenom {
            denom: denom.clone(),
        });
    }
    let balance = holding_response.balances.first().ok_or(InvalidDenom {
        denom: denom.clone(),
    })?;
    if balance.coins.len() != 1 {
        return Err(InvalidDenom {
            denom: denom.clone(),
        });
    }
    Ok(HeldCoin {
        coin: balance.coins.first().unwrap().clone(),
        address: balance.clone().address,
    })
}

pub fn is_seller(deps: &DepsMut, info: &MessageInfo) -> Result<bool, ContractError> {
    match retrieve_optional_seller_state(deps.storage)? {
        None => Ok(false),
        Some(seller) => Ok(seller.seller_address == info.sender),
    }
}

pub fn is_dealer(deps: &DepsMut, info: &MessageInfo) -> Result<bool, ContractError> {
    let config = retrieve_contract_config(deps.storage)?;
    Ok(config.dealers.contains(&info.sender))
}

pub fn seller_has_finalized(deps: &DepsMut) -> Result<bool, ContractError> {
    match retrieve_optional_seller_state(deps.storage)? {
        None => Ok(false),
        Some(seller) => Ok(!seller.pool_denoms.is_empty()),
    }
}

pub fn is_buyer(deps: &DepsMut, info: &MessageInfo) -> Result<bool, ContractError> {
    return match retrieve_optional_buyer_state(deps.storage)? {
        None => Ok(false),
        Some(state) => Ok(state.buyer_address == info.sender),
    };
}

pub fn buyer_has_accepted(deps: &DepsMut) -> Result<bool, ContractError> {
    return match retrieve_optional_buyer_state(deps.storage)? {
        None => Ok(false),
        Some(state) => Ok(state.buyer_has_accepted_pools),
    };
}

pub fn is_contract_admin(deps: &DepsMut, info: &MessageInfo) -> Result<bool, ContractError> {
    let config = retrieve_contract_config(deps.storage)?;
    return Ok(info.sender == config.contract_admin);
}

pub fn update_config_as_admin(
    deps: DepsMut,
    info: MessageInfo,
    updated_config: Config,
) -> Result<Response, ContractError> {
    if !is_contract_admin(&deps, &info)? {
        return Err(UnauthorizedConfigUpdate);
    }

    let seller_state = retrieve_optional_seller_state(deps.storage)?;
    let buyer_state = retrieve_optional_buyer_state(deps.storage)?;
    if seller_state.is_some() && buyer_state.is_some() {
        return Err(IllegalConfigUpdate)
    }

    save_contract_config(deps.storage, &updated_config)?;

    Ok(Response::new().add_attribute("contract_config", format!("{:?}", updated_config)))
}

pub fn create_send_coin_back_to_seller_messages(
    deps: &DepsMut,
    contract_address: String,
    seller_address: String,
    pool_denoms: Vec<String>,
) -> Result<Vec<MsgTransferRequest>, ContractError> {
    let mut messages = vec![];

    // Iterate over the list of denoms so that we can update the value owner of the scope to be the
    // seller instead of the contract
    for denom in pool_denoms {
        let held_coin = get_balance(&deps, denom.clone())?;

        // Transfer the coins back to the seller
        messages.push(MsgTransferRequest {
            amount: Some(held_coin.coin),
            administrator: contract_address.to_string(),
            from_address: contract_address.to_string(),
            to_address: seller_address.to_string(),
        });
    }
    return Ok(messages);
}

pub struct HeldCoin {
    pub coin: Coin,
    pub address: String,
}
