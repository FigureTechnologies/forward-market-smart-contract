use crate::error::ContractError;
use crate::error::ContractError::{InvalidEmptyDealerConfig, InvalidVisibilityConfig};
use crate::msg::InstantiateContractMsg;
use crate::storage::state_store::{save_bid_list_state, save_contract_config, BidList, Config};
use crate::util::helpers::validate_face_values;
use crate::version_info::{set_version_info, VersionInfoV1, CRATE_NAME, PACKAGE_VERSION};
use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo, Response};

pub fn instantiate_contract(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateContractMsg,
) -> Result<Response, ContractError> {
    // Validate the tick size against the min and max values
    validate_face_values(
        msg.min_face_value_cents,
        msg.max_face_value_cents,
        msg.tick_size,
    )?;

    // Convert the list of allowed sellers to addresses if the contract uses private sellers
    let mut allowed_sellers = vec![];
    if msg.use_private_sellers {
        allowed_sellers = validate_and_map_address(msg.allowed_sellers, &deps)?;
    } else if !msg.allowed_sellers.is_empty() {
        return Err(InvalidVisibilityConfig);
    }

    // Convert the list of allowed buyers to addresses if the contract uses private buyers
    let mut allowed_buyers = vec![];
    if msg.use_private_sellers {
        allowed_buyers = validate_and_map_address(msg.allowed_buyers, &deps)?;
    } else if !msg.allowed_buyers.is_empty() {
        return Err(InvalidVisibilityConfig);
    }

    // A dealer is required to initiate the transfer so make sure at least one is defined
    if msg.dealers.is_empty() {
        return Err(InvalidEmptyDealerConfig);
    }

    // Convert the list of dealers to addresses
    let dealer_addresses = validate_and_map_address(msg.dealers, &deps)?;

    // Store the initial configuration
    let config = Config {
        use_private_sellers: msg.use_private_sellers,
        use_private_buyers: msg.use_private_buyers,
        allowed_sellers,
        allowed_buyers,
        max_bid_count: msg.max_buyer_count,
        token_denom: msg.token_denom,
        max_face_value_cents: msg.max_face_value_cents,
        min_face_value_cents: msg.min_face_value_cents,
        tick_size: msg.tick_size,
        dealers: dealer_addresses,
        is_disabled: false,
        contract_admin: info.sender,
    };
    save_contract_config(deps.storage, &config)?;
    save_bid_list_state(deps.storage, &BidList { bids: vec![] })?;

    set_version_info(
        deps.storage,
        &VersionInfoV1 {
            version: PACKAGE_VERSION.to_string(),
            definition: CRATE_NAME.to_string(),
        },
    )?;

    Ok(Response::new().add_attribute("contract_config", format!("{:?}", config)))
}

fn validate_and_map_address(
    address_strings: Vec<String>,
    deps: &DepsMut,
) -> Result<Vec<Addr>, ContractError> {
    let mut addresses = vec![];
    for seller_str in address_strings {
        let seller_addr = deps.api.addr_validate(&seller_str)?;
        addresses.push(seller_addr)
    }
    return Ok(addresses);
}
