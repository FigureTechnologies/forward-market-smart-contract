use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The msg that is sent to the chain in order to instantiate a new instance of this contract's
/// stored code.  Used in the functionality defined in [instantiate_contract](crate::instantiate::instantiate_contract::instantiate_contract).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateContractMsg {
    /// A flag indicating whether to limit the allowed seller addresses to the list defined in the allowed sellers list
    pub is_private: bool,
    /// A list addresses allowed to be a seller in the contract. This is only valid if the is_private field is set to
    /// true and must be empty when is_private is false
    pub allowed_sellers: Vec<String>,
    /// A hash generated from the agreement terms that are stored in block vault
    pub agreement_terms_hash: String,
    /// The denom of the marker that all seller assets with be transferred to upon successful confirmation by the dealer
    pub token_denom: String,
    /// The maximum value that may be accepted by a seller
    pub max_face_value_cents: Uint128,
    /// The minimum value that may be accepted by a seller
    pub min_face_value_cents: Uint128,
    /// The number of coins per accepted_value_cents (if the seller accepts 1000 cents and tick size is 10, 100 coins
    /// will be minted for the token_denom)
    pub tick_size: Uint128,
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
        agreement_terms_hash: String,
    },
    /// A route that allows the sender to remove themselves from the list of allowed sellers
    RemoveAsSeller {},
    /// A route that allows the seller to finalize a list of pools
    FinalizePools {},
    /// A route executed by the dealer that causes the settlement of the transaction
    DealerConfirm {},
    /// A route that can be used by the buyer to update terms of the contract before a seller has been added
    UpdateAgreementTermsHash { agreement_terms_hash: String },
    /// A route that can be used by the buyer to update the face values before a seller has been added
    UpdateFaceValueCents {
        max_face_value_cents: Uint128,
        min_face_value_cents: Uint128,
        tick_size: Uint128,
    },
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
    pub buyer: BuyerResponse,
    pub seller: Option<SellerResponse>,
    pub config: ConfigResponse,
    pub settlement_data: Option<SettlementDataResponse>,
    pub version_info: VersionInfoResponse,
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SellerResponse {
    pub seller_address: Addr,
    pub accepted_value_cents: Uint128,
    pub pool_denoms: Vec<String>,
    pub offer_hash: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BuyerResponse {
    pub buyer_address: Addr,
    pub has_accepted_pools: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SettlementDataResponse {
    pub block_height: u64,
    pub settling_dealer: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VersionInfoResponse {
    pub definition: String,
    pub version: String,
}
