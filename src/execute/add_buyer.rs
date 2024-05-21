use crate::error::ContractError;
use crate::error::ContractError::{MaxPrivateBuyersReached, UnauthorizedPrivateBuyer};
use crate::storage::state_store::{
    retrieve_buyer_state, retrieve_contract_config, save_buyer_state, Buyer, BuyerList,
};
use cosmwasm_std::{DepsMut, MessageInfo, Response};

pub fn execute_add_buyer(
    deps: DepsMut,
    info: MessageInfo,
    agreement_terms_hash: String,
) -> Result<Response, ContractError> {
    let config = retrieve_contract_config(deps.storage)?;
    let buyer_state = retrieve_buyer_state(deps.storage)?;

    // If using private buyers, make sure this buyer is allowed to submit a bid
    if config.use_private_buyers {
        if !config.allowed_buyers.contains(&info.sender) {
            return Err(UnauthorizedPrivateBuyer);
        }

        if buyer_state.buyers.len() >= usize::try_from(config.max_buyer_count).unwrap() {
            return Err(MaxPrivateBuyersReached);
        }
    }

    let buyer_state = retrieve_buyer_state(deps.storage)?;

    // Remove any existing bid for this buyer because if one exists we want to replace it
    let mut buyers: Vec<Buyer> = buyer_state
        .buyers
        .into_iter()
        .filter(|buyer| buyer.buyer_address != info.sender)
        .collect();
    buyers.push(Buyer {
        buyer_address: info.sender,
        agreement_terms_hash,
    });

    // Save the updated buyer state
    save_buyer_state(
        deps.storage,
        &BuyerList {
            buyers: buyers.clone(),
        },
    )?;

    Ok(Response::new().add_attribute("buyers_list", format!("{:?}", buyers.clone())))
}
