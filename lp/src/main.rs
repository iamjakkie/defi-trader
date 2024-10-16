use alloy::providers::{Provider, ProviderBuilder, RootProvider, WsConnect};
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

#[tokio::main]
async fn main() {
    let rpc_url = env!("RPC_URL");
    let ws = WsConnect::new(rpc_url);
    let provider = match ProviderBuilder::new().on_ws(ws).await {
        Ok(provider) => provider,
        Err(e) => {
            eprintln!("Error connecting to provider: {:?}", e);
            return;
        }
    };

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

    }

