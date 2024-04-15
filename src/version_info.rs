use crate::error::ContractError;
use cosmwasm_std::Storage;
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const CRATE_NAME: &str = env!("CARGO_CRATE_NAME");
pub const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");
const VERSION_INFO_NAMESPACE: &str = "version_info";
const VERSION_INFO: Item<VersionInfoV1> = Item::new(VERSION_INFO_NAMESPACE);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VersionInfoV1 {
    pub definition: String,
    pub version: String,
}

pub fn get_version_info(store: &dyn Storage) -> Result<VersionInfoV1, ContractError> {
    VERSION_INFO.load(store).map_err(ContractError::Std)
}

pub fn set_version_info(
    store: &mut dyn Storage,
    version_info: &VersionInfoV1,
) -> Result<(), ContractError> {
    VERSION_INFO
        .save(store, version_info)
        .map_err(ContractError::Std)
}
