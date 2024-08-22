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

* `use_private_sellers`: A flag indicating whether to limit the allowed seller addresses to the list defined in the allowed sellers list
* `use_private_buyers`: A flag indicating whether to limit the allowed buyer addresses to the list defined in the allowed buyers list
* `allowed_sellers`: A list of addresses allowed to be a seller in the contract. This is only valid if the use_private_sellers field is set to true and must be empty when use_private_sellers is false
* `allowed_buyers`: A list of addresses allowed to be a buyer in the contract. This is only valid if the use_private_buyers field is set to true and must be empty when use_private_buyers is false
* `max_buyer_count`: The maximum number of bids that can be placed on the contract
* `dealers`: The list of addresses allowed to confirm and reset the contract

Example instantiation payload:
```json
{
  "use_private_sellers": true,
  "use_private_buyers": true,
  "allowed_sellers": ["mockpbselleraddress"],
  "allowed_buyers": ["mockpbbuyeraddress"],
  "max_buyer_count": 10,
  "dealers": ["mockpbdealeraddress"]
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
    "offer_hash": "b1c2d3e4"
  }
}

```

### UpdateSellerOfferHash
#### Allows the buyer to update terms of the contract before a seller has been added

* `offer_hash`: A hash generated from the offer terms that are stored in block vault

Example execution payload:

```json
{
  "UpdateSellerOfferHash": {
    "offer_hash": "a1b2c3d4"
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
    "pool_denoms": ["example.test.pool.0"]
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

### UpdateAllowedSellers
#### Allows the buyer to update the allowed seller's list before a seller has been added

* `allowed_sellers`: A list addresses allowed to be a seller in the contract. This is only valid if the is_private field is set to true and must be empty when is_private is false

Example execution payload:

```json
{
  "UpdateAllowedSellers": {
    "allowed_sellers": ["mockpbselleraddress"]
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

### ContractDisable
#### Allows the dealer to disable a contract provided that the contract does not hold any coins

Example execution payload:

```json
{
  "ContractDisable": {}
}
```

### AcceptBid
#### Allows the seller to accept one of the bids from the bid list

* `bidder_address`: The address of the bidder for the bid the seller wishes to accept
* `agreement_terms_hash`: The hash of the terms that the seller is agreeing to that are stored in block vault

Example execution payload:

```json
{
  "bidder_address": "mockpbbidderaddress",
  "agreement_terms_hash": "1d3d5d7"
}
```

### AddBid
#### Allows a potential buyer to add a bid to the bid list

* `agreement_terms_hash`: A hash generated from the agreement terms that are stored in block vault

Example execution payload:

```json
{
  "agreement_terms_hash": "2j547d5e"
}
```

### MintTokens
#### Allows the admin of the contract to mint the tokens that will be given to the buyer when their bid is accepted

* `token_count`: The number of tokens that will be minted for the specified denom
* `token_denom`: The denom of the marker that will hold the tokens

Example execution payload:

```json
{
  "token_count": "5000",
  "token_denom": "test.mock.fake.denom"
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
