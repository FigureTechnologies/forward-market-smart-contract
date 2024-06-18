use crate::storage::state_store::{Bid, Config, Seller, SettlementData, TransactionState};
use crate::version_info::VersionInfoV1;
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The msg that is sent to the chain in order to instantiate a new instance of this contract's
/// stored code.  Used in the functionality defined in [instantiate_contract](crate::instantiate::instantiate_contract::instantiate_contract).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateContractMsg {
    /// A flag indicating whether to limit the allowed seller addresses to the list defined in the allowed sellers list
    pub use_private_sellers: bool,
    /// A flag indicating whether to limit the allowed buyer addresses to the list defined in the allowed buyers list
    pub use_private_buyers: bool,
    /// A list of addresses allowed to be a seller in the contract. This is only valid if the use_private_sellers field is set to
    /// true and must be empty when use_private_sellers is false
    pub allowed_sellers: Vec<String>,
    /// A list of addresses allowed to be a buyer in the contract. This is only valid if the use_private_buyers field is set to
    /// true and must be empty when use_private_buyers is false
    pub allowed_buyers: Vec<String>,
    /// The max number of potential buyers allowed to submit bids to the contract
    pub max_buyer_count: i32,
    /// The denom of the marker that all seller assets with be transferred to upon successful confirmation by the dealer
    pub token_denom: String,
    /// The number of coins that will represent the forward market
    pub token_count: Uint128,
    /// The list of addresses allowed to confirm and reset the contract
    pub dealers: Vec<String>,
}

/// All defined payloads to be used when executing routes on this contract instance.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum ExecuteMsg {
    /// A route that adds the sender as the seller on the contract
    AddSeller {
        accepted_value_cents: Uint128,
        offer_hash: String,
    },
    /// A route that allows the sender to remove themselves from the list of allowed sellers
    RemoveAsSeller {},
    /// A route that allows the seller to finalize a list of pools
    FinalizePools { pool_denoms: Vec<String> },
    /// A route executed by the dealer that causes the settlement of the transaction
    DealerConfirm {},
    /// A route that can be used by the buyer to update the allowed seller's list before a seller has been added
    UpdateAllowedSellers { allowed_sellers: Vec<String> },
    /// A route used by the buyer to accept a seller's finalized list of pools
    AcceptFinalizedPools {},
    /// A route used by the seller to rescind a finalized list of pools before the buyer has accepted
    RescindFinalizedPools {},
    /// A route used by the dealer to reset a contract, which will clear buyer acceptance, seller finalization, and
    /// return the coins in escrow by the contract back to the seller
    DealerReset {},
    /// A route used by either the buyer or a dealer to disable the contract. The seller must not have a
    /// finalized list of pools in order for the contract to be disabled (if the seller does have a
    /// finalized list of pools, either the seller must rescind the offer or a dealer must reset the
    /// contract before the disable operation will be allowed).
    ContractDisable {},
    /// A route used by the seller to accept a bid from a buyer in the list of buyer bids
    AcceptBid {
        bidder_address: String,
        agreement_terms_hash: String,
    },
    /// A route used by a potential buyer to add their bid to the list of buyer bids
    AddBid { agreement_terms_hash: String },
}

/// All defined payloads to be used when querying routes on this contract instance.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, QueryResponses)]
pub enum QueryMsg {
    /// A route used to ready the internal state of the contract
    #[returns(GetContractStateResponse)]
    GetContractState {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GetContractStateResponse {
    pub buyers: Vec<Bid>,
    pub seller: Option<Seller>,
    pub config: Config,
    pub settlement_data: Option<SettlementData>,
    pub version_info: VersionInfoV1,
    pub transaction_state: Option<TransactionState>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MetadataAddress {
    pub bech32: String,
    pub bytes: Vec<u8>,
    pub key_type: KeyType,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum KeyType {
    Scope = 0x00,
    Session = 0x01,
    Record = 0x02,
    ContractSpecification = 0x03,
    ScopeSpecification = 0x04,
    RecordSpecification = 0x05,
}

impl KeyType {
    pub fn to_str(&self) -> &str {
        match self {
            KeyType::Scope => "scope",
            KeyType::Session => "session",
            KeyType::Record => "record",
            KeyType::ContractSpecification => "contractspec",
            KeyType::ScopeSpecification => "scopespec",
            KeyType::RecordSpecification => "recspec",
        }
    }
}
