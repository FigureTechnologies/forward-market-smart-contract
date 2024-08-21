use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    /// Occurs when an error is encountered during contract store communication.
    #[error("Contract storage error occurred: {message}")]
    StorageError {
        /// A message describing the nature of the error.
        message: String,
    },

    /// Occurs when an action must be performed by the seller but is attempted by a different address
    #[error("Action must be performed by seller")]
    UnauthorizedAsSeller,

    /// Occurs when a seller is attempted to be registered when a seller is already registered
    #[error("A seller has already been defined for this contract")]
    SellerAlreadyExists,

    /// Occurs when a seller that is not specified in the allowed sellers list tries to add themselves as seller
    #[error("Seller address is not in allowed list of private sellers")]
    UnauthorizedPrivateSeller,

    /// Occurs when a buyer that is not specified in the allowed buyers list tries to submit a bid
    #[error("Buyer address is not in allowed list of private buyers")]
    UnauthorizedPrivateBuyer,

    /// Occurs if anyone other than the contract admin attempts to modify contract configuration options
    #[error("Only the contract admin can update the contract configuration")]
    UnauthorizedConfigUpdate,

    /// Occurs if anyone other than the contract admin attempts to mint the tokens for the contract
    #[error("Only the contract admin can mint the tokens for the forward market transaction")]
    UnauthorizedToMint,

    /// Occurs if the contract admin attempts to modify configuration after a buyer and seller have been added
    #[error("Configuration cannot be updated once a buyer and seller have been established")]
    IllegalConfigUpdate,

    /// Occurs if the seller tries to finalize without adding any asset pools
    #[error("The set of pools cannot be empty when finalizing the transaction")]
    InvalidFinalizationRequest,

    /// Occurs if the list of approved addresses is not empty while the visibility config for that list is private
    #[error("The list of approved addresses should be empty unless the visibility config for that list is private ")]
    InvalidVisibilityConfig,

    /// Occurs if a face value or accepted value cannot be divided without remainder by the tick value
    #[error("Token count must be greater than 0")]
    InvalidTokenCount,

    /// Occurs if an attempt to finalize a list of pools is made after the pool has already been finalized by the seller
    #[error("The list of pools has already been finalized")]
    PoolAlreadyFinalized,

    /// Occurs if an attempt to accept a list of pools is made after the pool has already been accepted by the buyer
    #[error("The list of pools has already been accepted")]
    PoolAlreadyAccepted,

    /// Occurs if the buyer attempts to accept before a seller has provided a list of pools
    #[error(
        "The list of pools must be finalized by the seller before it can be accepted by the buyer"
    )]
    IllegalPoolAcceptanceRequest,

    /// Occurs if someone other than the buyer attempts to finalize the list of pools
    #[error("Only the buyer can accept the finalized list of pools")]
    IllegalAcceptingParty,

    /// Occurs if the dealer attempts to confirm the contract before both parties have agreed to the contract
    #[error("The seller must finalize the pool list and the buyer must accept it before confirmation is allowed")]
    InvalidConfirmationRequest,

    /// Occurs if a denom in the list of pools is not valid
    #[error("Invalid denom specified {denom:?}")]
    InvalidDenom { denom: String },

    /// Occurs if the seller attempts to finalize with a denom for which they do not own all coins
    #[error(
        "Invalid denom ownership detected. Contract only supports one owner per denom: {denom:?}"
    )]
    InvalidDenomOwnership { denom: String },

    /// Occurs if the seller provides a denom that cannot be found on chain
    #[error("Marker base account address not found: {denom:?}")]
    MissingMarkerBaseAccount { denom: String },

    /// Occurs if a seller tries to remove themselves from the list of sellers when a contract is not marked private
    #[error("Seller removal request is not applicable to a public forward market contract")]
    InvalidSellerRemovalRequest,

    /// Occurs if the seller tries to remove themselves from the list of allowed sellers when they are not part of
    /// the list
    #[error("Seller must be part of allowed seller list in order to remove themselves")]
    IllegalSellerRemovalRequest,

    /// Occurs if the seller attempts to remove themselves from the list of allowed sellers after they have already
    /// been added as the seller
    #[error("Seller cannot remove themselves from the accepted list if they are already designated as the seller of the contract")]
    SellerAlreadyAccepted,

    /// Occurs if the seller attempts to submit a pool of loans that the seller does not own
    #[error("Only coin owned by the seller can be added to the list of proposed pools")]
    IllegalCoinOwnership,

    /// Occurs if the seller attempts to rescind their offer after the buyer has already accepted
    #[error(
        "The finalized pool cannot be rescinded once the buyer has accepted the list of pools"
    )]
    IllegalRescindRequest,

    /// Occurs if the seller attempts to rescind before the seller has finalized a list of pools
    #[error("Only a finalized pool can be rescinded")]
    InvalidRescindRequest,

    /// Occurs if the buyer attempts to configure the dealer list as empty
    #[error("The list of dealers cannot be empty")]
    InvalidEmptyDealerConfig,

    /// Occurs if someone other than the dealer attempts to confirm the transaction
    #[error("Only a dealer can confirm the transaction")]
    IllegalConfirmationRequest,

    /// Occurs if someone other than the dealer attempts to reset a transaction
    #[error("Only a dealer can reset a transaction")]
    IllegalDealerResetRequest,

    /// Occurs if a dealer attempts to reset a contract before a seller has been added to the contract
    #[error("A reset cannot be performed when no seller has been added to the contract")]
    InvalidDealerResetRequest,

    /// Occurs if someone other than the buyer attempts to run the access list migration
    #[error("A reset cannot be performed when no seller has been added to the contract")]
    UnauthorizedAccessListMigrationRequest,

    /// Occurs if an execution method is called after a contract has already been settled
    #[error("Contract execution methods are not allowed after a contract has been settled")]
    IllegalContractExecution,

    /// Occurs if an execution method is called when the contract has been disabled
    #[error("Contract execution methods cannot be executed once the contract has been disabled")]
    InvalidContractExecution,

    /// Occurs if either the buyer or a dealer attempts to disable a contract with a buyer that
    /// has finalized an offer
    #[error("The contract cannot be disabled while the seller has a finalized list of pools")]
    IllegalDisableRequest,

    /// Occurs if a party other than the buyer or a dealer attempts to disable a contract
    #[error("A contract may only be disabled by the buyer or a dealer")]
    UnauthorizedDisableRequest,

    /// Occurs if the seller is agreeing to a terms hash that does not match the latest for the buyer
    #[error("The agreement terms hash provided by the seller does not match the current agreement terms hash")]
    InvalidAgreementTermsHash,

    /// Occurs if the buyer is agreeing to an offer hash that does not match the latest for the seller
    #[error("The offer hash provided does not match the current offer hash of the seller")]
    InvalidOfferHash,

    /// Occurs when a buyer attempts to submit a bid but the limit of allowed buyers has already been reached
    #[error("The limit of allowed buyers has already been reached")]
    MaxPrivateBuyersReached,

    /// Occurs when a seller attempts to accept a bid for a bidder address that doesn't exist
    #[error("Bid does not exist for address {address:?}")]
    BidDoesNotExist { address: String },

    /// Occurs when a seller attempts to accept a bid when a previous bid has already been accepted
    #[error("Cannot accept bid because a bid from address {address:?} was already accepted")]
    BidPreviouslyAccepted { address: String },

    /// Occurs when a seller attempts to update an offer hash after a buyer has accepted the offer
    #[error("The offer has cannot be updated after a buyer has accepted it")]
    IllegalOfferHashUpdate,

    /// Occurs when tokens for the contract have previously been minted
    #[error("Tokens for the contract were already minted for token denom {token_denom:?}")]
    TokensAlreadyMinted { token_denom: String },

    /// Occurs when a seller tries to accept a bid before the tokens have been minted
    #[error("Bid cannot be accepted until the admin has completed the MintTokens action")]
    TokensNotMinted,

    /// Occurs when a migration is attempted for an unsupported version
    #[error("Migration does not support {version:?} version")]
    IllegalMigrationVersion { version: String },
}
