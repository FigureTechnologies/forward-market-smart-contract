use crate::error::ContractError;
use crate::error::ContractError::{InvalidEmptyDealerConfig, InvalidVisibilityConfig};
use crate::msg::InstantiateContractMsg;
use crate::storage::state_store::{save_buyer_state, save_contract_config, Buyer, Config};
use crate::util::helpers::validate_face_values;
use crate::version_info::{set_version_info, VersionInfoV1, CRATE_NAME, PACKAGE_VERSION};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

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

    // Convert the list of allowed sellers to addresses if the contract is private
    let mut allowed_sellers = vec![];
    if msg.is_private {
        for seller_str in msg.allowed_sellers {
            let seller_addr = deps.api.addr_validate(&seller_str)?;
            allowed_sellers.push(seller_addr)
        }
    } else if !msg.allowed_sellers.is_empty() {
        return Err(InvalidVisibilityConfig);
    }

    // Convert the list of dealers to addresses
    let mut dealer_addresses = vec![];
    for dealer in msg.dealers {
        let dealer_addr = deps.api.addr_validate(&dealer)?;
        dealer_addresses.push(dealer_addr)
    }

    // A dealer is required to initiate the transfer so make sure at least one is defined
    if dealer_addresses.is_empty() {
        return Err(InvalidEmptyDealerConfig);
    }

    // Store the initial configuration
    let config = Config {
        is_private: msg.is_private,
        allowed_sellers,
        agreement_terms_hash: msg.agreement_terms_hash,
        token_denom: msg.token_denom,
        max_face_value_cents: msg.max_face_value_cents,
        min_face_value_cents: msg.min_face_value_cents,
        tick_size: msg.tick_size,
        dealers: dealer_addresses,
        is_disabled: false,
    };
    save_contract_config(deps.storage, &config)?;

    // Store the buyer address
    let buyer = Buyer {
        buyer_address: info.sender.clone(),
        has_accepted_pools: false,
    };
    save_buyer_state(deps.storage, &buyer)?;

    set_version_info(
        deps.storage,
        &VersionInfoV1 {
            version: PACKAGE_VERSION.to_string(),
            definition: CRATE_NAME.to_string(),
        },
    )?;

    Ok(Response::new().add_attribute("contract_config", format!("{:?}", config)))
}
