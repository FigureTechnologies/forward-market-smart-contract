use crate::error::ContractError;
use crate::error::ContractError::StorageError;
use cosmwasm_std::{Addr, Storage, Uint128};
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

pub fn retrieve_buyer_state(storage: &dyn Storage) -> Result<BuyerList, ContractError> {
    BUYERS.load(storage).map_err(|e| StorageError {
        message: format!("{e:?}"),
    })
}

pub fn save_buyer_state(storage: &mut dyn Storage, buyer: &BuyerList) -> Result<(), ContractError> {
    BUYERS.save(storage, buyer).map_err(|e| StorageError {
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

pub fn retrieve_optional_transaction_state(
    storage: &dyn Storage,
) -> Result<Option<TransactionState>, ContractError> {
    TRANSACTION_STATE
        .may_load(storage)
        .map_err(|e| StorageError {
            message: format!("{e:?}"),
        })
}

pub fn save_transaction_state(
    storage: &mut dyn Storage,
    transaction_state: &TransactionState,
) -> Result<(), ContractError> {
    TRANSACTION_STATE
        .save(storage, transaction_state)
        .map_err(|e| StorageError {
            message: format!("{e:?}"),
        })
}

pub fn clear_transaction_state(storage: &mut dyn Storage) -> () {
    TRANSACTION_STATE.remove(storage)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Config {
    pub use_private_sellers: bool,
    pub use_private_buyers: bool,
    pub allowed_sellers: Vec<Addr>,
    pub allowed_buyers: Vec<Addr>,
    pub max_buyer_count: i32,
    pub token_denom: String,
    pub max_face_value_cents: Uint128,
    pub min_face_value_cents: Uint128,
    pub tick_size: Uint128,
    pub dealers: Vec<Addr>,
    pub is_disabled: bool,
    pub contract_admin: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Seller {
    pub seller_address: Addr,
    pub accepted_value_cents: Uint128,
    pub pool_denoms: Vec<String>,
    pub offer_hash: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct BuyerList {
    pub buyers: Vec<Buyer>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Buyer {
    pub buyer_address: Addr,
    pub agreement_terms_hash: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct SettlementData {
    pub block_height: u64,
    pub settling_dealer: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct TransactionState {
    pub buyer_address: Addr,
    pub buyer_has_accepted_pools: bool,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const SELLER: Item<Seller> = Item::new("seller");
pub const BUYERS: Item<BuyerList> = Item::new("buyer_list");
pub const SETTLEMENT_DATA: Item<SettlementData> = Item::new("settlement_data");
pub const TRANSACTION_STATE: Item<TransactionState> = Item::new("transaction_state");
