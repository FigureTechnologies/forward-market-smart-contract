use crate::error::ContractError;
use crate::error::ContractError::IllegalMigrationVersion;
use crate::version_info::{get_version_info, set_version_info, VersionInfoV1, PACKAGE_VERSION};
use cosmwasm_std::{DepsMut, Response};

pub fn migrate_contract(deps: DepsMut) -> Result<Response, ContractError> {
    let current_version_info = get_version_info(deps.storage)?;
    validate_migration(current_version_info.version)?;
    set_version_info(
        deps.storage,
        &VersionInfoV1 {
            definition: current_version_info.definition,
            version: PACKAGE_VERSION.to_string(),
        },
    )?;
    Ok(Response::new()
        .add_attribute("action", "migrate")
        .add_attribute("new_version", PACKAGE_VERSION.to_string()))
}

fn validate_migration(current_version: String) -> Result<(), ContractError> {
    if current_version != "0.1.1" {
        return Err(IllegalMigrationVersion {
            version: current_version,
        });
    }
    Ok(())
}
