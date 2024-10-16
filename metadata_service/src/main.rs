use alloy::providers::{Provider, ProviderBuilder, RootProvider};
use alloy::sol;
use std::env;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use alloy::eips::{BlockId, BlockNumberOrTag};
use alloy::json_abi::InternalType::Contract;
use alloy::primitives::{address, Address, U256};
use alloy::providers::network::primitives::BlockTransactionsKind::Full;
use alloy::transports::http::{Client, Http};
use tokio_tungstenite::{accept_async, connect_async, tungstenite::Message};
use futures_util::sink::SinkExt;
use tokio::main;
use tokio::net::TcpListener;
use zmq::{Context, SUB};

sol!(
    #[sol(rpc)]
    IERC20,
    "../artifacts/ERC20.json"
);

#[derive(Debug)]
struct Token {
    address: Address,
    name: String,
    symbol: String,
    decimals: u8,
    supply: U256,
}

#[tokio::main]
async fn main() {
    let rpc_url = env!("RPC_URL");
    let provider = ProviderBuilder::new().on_http(rpc_url.parse().unwrap());


    // println!("name: {}", name);
    // let IERC20::decimalsReturn { decimals } = c.decimals().call().await.unwrap();
    // println!("decimals: {}", decimals);
// }

    let context = Context::new();
    let subscriber = context.socket(SUB).unwrap();
    subscriber.connect("tcp://localhost:5555").unwrap();

    // Subscribe to all messages ("" means no filter)
    subscriber.set_subscribe(b"").unwrap();

    loop {
        let message = subscriber.recv_string(0).unwrap().unwrap();
        println!("Metadata Service received: {}", message);

        let contract_address = Address::from_str(&message).unwrap();
        // Clone the provider inside the async block
        let provider_clone = provider.clone();

        let mut contract = IERC20::new(contract_address, provider_clone);

        // Spawn a new async block to handle the contract interaction
        tokio::spawn(async move {
            let name = contract.clone().name().call().await.unwrap();
            let symbol = contract.clone().symbol().call().await.unwrap();
            let decimals = contract.clone().decimals().call().await.unwrap();
            let total_supply = contract.clone().totalSupply().call().await.unwrap();

            let token = Token {
                address: contract_address,
                name: name._0,
                symbol: symbol._0,
                decimals: decimals._0,
                supply: total_supply._0,
            };

            println!("Token: {:?}", token);
        });
    }
}