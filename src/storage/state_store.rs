use crate::error::ContractError;
use crate::error::ContractError::StorageError;
use cosmwasm_std::{Addr, Coin, Storage, Uint128};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub fn retrieve_contract_config(storage: &dyn Storage) -> Result<Config, ContractError> {
    CONFIG.load(storage).map_err(|e| StorageError {
        message: format!("{e:?}"),
    })
}

pub fn save_contract_config(
    storage: &mut dyn Storage,
    config: &Config,
) -> Result<(), ContractError> {
    CONFIG.save(storage, config).map_err(|e| StorageError {
        message: format!("{e:?}"),
    })
}

pub fn retrieve_buyer_state(storage: &dyn Storage) -> Result<Buyer, ContractError> {
    BUYER.load(storage).map_err(|e| StorageError {
        message: format!("{e:?}"),
    })
}

pub fn save_buyer_state(storage: &mut dyn Storage, buyer: &Buyer) -> Result<(), ContractError> {
    BUYER.save(storage, buyer).map_err(|e| StorageError {
        message: format!("{e:?}"),
    })
}

pub fn retrieve_optional_seller_state(
    storage: &dyn Storage,
) -> Result<Option<Seller>, ContractError> {
    SELLER.may_load(storage).map_err(|e| StorageError {
        message: format!("{e:?}"),
    })
}

pub fn retrieve_seller_state(storage: &dyn Storage) -> Result<Seller, ContractError> {
    SELLER.load(storage).map_err(|e| StorageError {
        message: format!("{e:?}"),
    })
}

pub fn save_seller_state(storage: &mut dyn Storage, seller: &Seller) -> Result<(), ContractError> {
    SELLER.save(storage, seller).map_err(|e| StorageError {
        message: format!("{e:?}"),
    })
}

pub fn retrieve_optional_settlement_data_state(
    storage: &dyn Storage,
) -> Result<Option<SettlementData>, ContractError> {
    SETTLEMENT_DATA.may_load(storage).map_err(|e| StorageError {
        message: format!("{e:?}"),
    })
}

pub fn save_settlement_data_state(
    storage: &mut dyn Storage,
    settlement_data: &SettlementData,
) -> Result<(), ContractError> {
    SETTLEMENT_DATA
        .save(storage, settlement_data)
        .map_err(|e| StorageError {
            message: format!("{e:?}"),
        })
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Config {
    pub is_private: bool,
    pub allowed_sellers: Vec<Addr>,
    pub agreement_terms_hash: String,
    pub token_denom: String,
    pub max_face_value_cents: Uint128,
    pub min_face_value_cents: Uint128,
    pub tick_size: Uint128,
    pub dealers: Vec<Addr>,
    pub is_disabled: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Seller {
    pub seller_address: Addr,
    pub accepted_value_cents: Uint128,
    pub pool_coins: Vec<Coin>,
    pub offer_hash: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Buyer {
    pub buyer_address: Addr,
    pub has_accepted_pools: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct SettlementData {
    pub block_height: u64,
    pub settling_dealer: Addr,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const SELLER: Item<Seller> = Item::new("seller");
pub const BUYER: Item<Buyer> = Item::new("buyer");
pub const SETTLEMENT_DATA: Item<SettlementData> = Item::new("settlement_data");
