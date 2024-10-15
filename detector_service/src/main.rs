use alloy::providers::{Provider, ProviderBuilder, WsConnect};
use alloy::sol;
use std::env;
use std::sync::{Arc, Mutex};
use alloy::eips::{BlockId, BlockNumberOrTag};
use alloy::providers::network::primitives::BlockTransactionsKind::Full;
use tokio_tungstenite::{accept_async, connect_async, tungstenite::Message};
use futures_util::sink::SinkExt;
use tokio::net::TcpListener;

sol!(
    #[sol(rpc)]
    IERC20,
    "../artifacts/ERC20.json"
);
#[tokio::main]
async fn main() {
    let rpc_url = env!("RPC_URL");

    // Set up WebSocket server on localhost:9001
    let ws_listener = TcpListener::bind("127.0.0.1:9001")
        .await
        .expect("Failed to bind WebSocket server");

    let connected_clients: Arc<Mutex<Vec<tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>>>> = Arc::new(Mutex::new(Vec::new()));

    let clients = connected_clients.clone();
    tokio::spawn(async move {
        while let Ok((stream, _)) = ws_listener.accept().await {
            let ws_stream = accept_async(stream)
                .await
                .expect("Error during WebSocket handshake");

            println!("New WebSocket connection established");

            clients.lock().unwrap().push(ws_stream);
        }
    });

    let ws = WsConnect::new(rpc_url);

    let provider = match ProviderBuilder::new().on_ws(ws).await {
        Ok(provider) => provider,
        Err(e) => {
            eprintln!("Error connecting to provider: {:?}", e);
            return;
        }
    };

    let mut current_block = BlockNumberOrTag::Latest;

    loop {
        match provider.get_block(BlockId::from(current_block), Full).await {
            Ok(Some(block)) => {
                println!("Processing block: {:?}", block.header.number);

                for tx in block.transactions.into_transactions() {
                    let dest = tx.to;
                    match dest {
                        None => {
                            // This is a contract creation transaction
                            let tx_hash = tx.hash;
                            let receipt = provider.get_transaction_receipt(tx_hash).await.unwrap().unwrap();
                            let contract_address = receipt.contract_address.unwrap();

                            let msg = format!("New contract created: {:?}", contract_address);

                            // Push the message to all connected clients
                            let mut clients = connected_clients.lock().unwrap();

                            // Create a new vector to store active clients
                            let mut active_clients = Vec::new();

                            for mut client in clients.drain(..) {
                                let result = client.send(Message::Text(msg.clone())).await;

                                match result {
                                    Ok(_) => {
                                        // If the message was sent successfully, keep the client in the list
                                        active_clients.push(client);
                                    }
                                    Err(_) => {
                                        // If there's an error (e.g., connection closed), print the error
                                        println!("WebSocket connection closed");
                                    }
                                }
                            }

                            // Replace the original list with the active clients
                            *clients = active_clients;

                        }
                        _ => {}
                    }
                }


                // Move to the next block
                current_block = BlockNumberOrTag::Number(block.header.number + 1);
                // break;
            }
            Ok(None) => {
                // If no block is found (possibly waiting for the next block)
                println!("Waiting for new block...");
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
            Err(e) => {
                eprintln!("Error fetching block: {:?}", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }
    }
}
