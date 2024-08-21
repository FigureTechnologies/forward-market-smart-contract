# Forward Market Smart Contract

This contract uses [Cosmwasm](https://github.com/CosmWasm/cosmwasm)'s provided architecture in
conjunction with [Provwasm](#https://github.com/provenance-io/provwasm) to create a wasm smart
contract that can be deployed to and interact with the Provenance Blockchain.

This contract provides functionality for creating a forward market contract for a single buyer and seller.
A contract is instantiated by a buyer and then accepted by a seller. A buyer may allow any seller or mark
the contract as private and only allow sellers of their choosing. The contract also requires a dealer, which
is a party that will trigger the settlement of the contract once both parties have accepted the terms of the
contract.

## Status
[![Latest Release][release-badge]][release-latest]
[![Apache 2.0 License][license-badge]][license-url]

[license-badge]: https://img.shields.io/github/license/FigureTechnologies/forward-market-smart-contract.svg
[license-url]: https://github.com/FigureTechnologies/forward-market-smart-contract/blob/main/LICENSE
[release-badge]: https://img.shields.io/github/tag/FigureTechnologies/forward-market-smart-contract.svg
[release-latest]: https://github.com/FigureTechnologies/forward-market-smart-contract/releases/latest

## Contract Instantiation

To instantiate a new instance of the contract, the following parameters are required:

* `is_private`: A flag indicating whether to limit the allowed seller addresses to the list defined in the allowed sellers list
* `allowed_sellers`: A list addresses allowed to be a seller in the contract. This is only valid if the is_private field is set to true and must be empty when is_private is false
* `agreement_terms_hash`: A hash generated from the agreement terms that are stored in block vault
* `token_denom`: The denom of the marker that all seller assets with be transferred to upon successful confirmation by the dealer
* `max_face_value_cents`: The maximum value that may be accepted by a seller
* `min_face_value_cents`: The minimum value that may be accepted by a seller
* `tick_size`: The number of coins per accepted cents by the seller (if the seller accepts 1000 cents and tick size is 10, 100 coins will be minted for the token_denom)
* `dealers`: The list of addresses allowed to confirm and reset the contract

Example instantiation payload:
```json
{
  "is_private": true,
  "allowed_sellers": listOf("mockpbselleraddress"),
  "agreement_terms_hash": "a1b2c3d4",
  "token_denom": "example.test.token.forward.market",
  "max_face_value_cents": 100_000_000,
  "min_face_value_cents": 50_000_000,
  "tick_size": 1000,
  "dealers": listOf("mockpbdealeraddress")
}
```

## Contract Execution
### AddSeller
#### Adds the sender as the seller on the contract. Along with the sender being added, a hash of the offer terms is added

* `offer_hash`: A hash generated from the offer terms that are stored in block vault

Example execution payload:

```json
{
  "AddSeller": {
    "offer_hash": "b1c2d3e4",
  }
}

```

### FinalizePools
#### Allows the seller to finalize a list of specified pools. This means that the buyer can now review and potentially accept the pools

* `pool_denoms`: The list of denoms for the markers that hold the pooled assets

Example execution payload:

```json
{
  "FinalizePools": {
    "pool_denoms": listOf("example.test.pool.0)
  }
}
```

### DealerConfirm
#### Allows the dealer to initiate the settlement of the transaction

Example execution payload:

```json
{
  "DealerConfirm": {}
}
```

### UpdateAgreementTermsHash
#### Allows the buyer to update terms of the contract before a seller has been added

* `agreement_terms_hash`: A hash generated from the agreement terms that are stored in block vault

Example execution payload:

```json
{
  "UpdateAgreementTermsHash": {
    "agreement_terms_hash": "a1b2c3d4"
  }
}
```

### UpdateFaceValueCents
#### Allows the buyer to update the face values before a seller has been added

* `max_face_value_cents`: The maximum value that may be accepted by a seller
* `min_face_value_cents`: The minimum value that may be accepted by a seller
* `tick_size`: The number of coins per accepted cents by the seller (if the seller accepts 1000 cents and tick size is 10, 100 coins will be minted for the token_denom)

Example execution payload:

```json
{
  "UpdateFaceValueCents": {
    "max_face_value_cents": 100_000_000,
    "min_face_value_cents": 50_000_000,
    "tick_size": 1000
  }
}
```

### UpdateAllowedSellers
#### Allows the buyer to update the allowed seller's list before a seller has been added

* `allowed_sellers`: A list addresses allowed to be a seller in the contract. This is only valid if the is_private field is set to true and must be empty when is_private is false

Example execution payload:

```json
{
  "UpdateAllowedSellers": {
    "allowed_sellers": listOf("mockpbselleraddress")
  }
}
```

### AcceptFinalizedPool
#### Allows the buyer to accept a seller's finalized list of pools

Example execution payload:

```json
{
  "AcceptFinalizedPool": {}
}
```

### RescindFinalizedPools
#### Allows the seller to rescind a finalized list of pools before the buyer has accepted

Example execution payload:

```json
{
  "RescindFinalizedPools": {}
}
```

### DealerReset
#### Allows the dealer to reset a contract, which will clear buyer acceptance, seller finalization, and return the coins in escrow by the contract back to the seller

Example execution payload:

```json
{
  "DealerReset": {}
}
```

### ContractDisable
#### Allows the dealer to disable a contract provided that the contract does not hold any coins

```json
{
  "ContractDisable": {}
}
```

## Contract Query

The contract currently provides a single query route for getting its internal state. It can be queried
with the following payload:

```json
{
  "GetContractState": {}
}
```

## Development Setup
This assumes the user is running Mac OSX.

- To start developing with Rust, follow the standard [guide](https://www.rust-lang.org/tools/install).
- The contract uses `wasm-pack` with its `make build` command.  Use this [installer command](https://rustwasm.github.io/wasm-pack/installer/) to install it.
- To build the contract locally with its `make optimize`, a [Docker Environment](https://www.docker.com/products/docker-desktop/) is also required.
