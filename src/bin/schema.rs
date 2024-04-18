use cosmwasm_schema::write_api;
use forward_market_contract::msg::{ExecuteMsg, InstantiateContractMsg};

fn main() {
    write_api! {
        instantiate: InstantiateContractMsg,
        execute: ExecuteMsg,
    }
}
