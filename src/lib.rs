//! Forward Market Smart Contract
//!
//! This contract uses [Cosmwasm](https://github.com/CosmWasm/cosmwasm)'s provided architecture in
//! conjunction with [Provwasm](#https://github.com/provenance-io/provwasm) to create a wasm smart
//! contract that can be deployed to and interact with the Provenance Blockchain.
//!
//! This contract provides functionality for creating a forward market transaction.
//! A contract is instantiated by the contract admin. The accepting of a contract by the seller and
//! the adding of bids by potential buyers can happen in any order. When a seller has been added
//! and there are potential buyer bids, a seller can accept one of the bids. The contract also
//! requires a dealer, which is a party that will trigger the settlement of the contract once both
//! parties have accepted the terms of the contract.

/// The entry point of all commands sent to the contract
pub mod contract;
// The custom errors that the contract can return
pub mod error;
// All commands that can be executed by the contract
pub mod execute;
// Defines the contract instantiation
pub mod instantiate;
// Defines the messages that map to execution logic
pub mod msg;
// Defines the contract query process
mod query;
// Defines the state storage for the contract
mod storage;
// Defines the tests for the contract
pub mod tests;
// Utility methods that give access to shared logic
mod migrate;
pub mod util;
pub mod version_info;
