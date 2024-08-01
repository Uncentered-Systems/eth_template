use dotenvy::from_read;
use lazy_static::lazy_static;
use std::env;
use std::io::Cursor;

use chrono::Utc;
use kinode_process_lib::http::{bind_ws_path, send_ws_push, WsMessageType};
use kinode_process_lib::{
    await_message, call_init,
    eth::{EthConfigAction, NodeOrRpcUrl, Provider, ProviderConfig},
    get_blob,
    http::{self},
    println, Address, LazyLoadBlob, Message, Request, Response,
};
mod encryption;
mod eth_utils;
use eth_utils::Caller;
mod counter_caller;
use alloy::signers::{local::PrivateKeySigner, SignerSync};
use alloy_primitives::{Signature, U256};
use alloy_signer::{LocalWallet, Signer};
use counter_caller::CounterCaller;
mod types;
use std::collections::HashMap;
use std::str::FromStr;
use types::{Action, PrivateKey, SerializableWallet, State, WsPush, WsUpdate};

use crate::encryption::{decrypt_data, encrypt_data};

wit_bindgen::generate!({
    path: "target/wit",
    world: "process-v0",
});

lazy_static! {
    pub static ref CURRENT_CHAIN_ID: u64 = {
        let env_content = include_str!("../../../.env");
        from_read(Cursor::new(env_content)).expect("Failed to parse .env content");
        env::var("VITE_CURRENT_CHAIN_ID").expect("CHAIN_ID must be set").parse().unwrap()
    };

    pub static ref CONTRACT_ADDRESS: String = {
        let env_content = include_str!("../../../.env");
        from_read(Cursor::new(env_content)).expect("Failed to parse .env content");
        match *CURRENT_CHAIN_ID {
            31337 => env::var("VITE_ANVIL_CONTRACT_ADDRESS").expect("CONTRACT_ADDRESS must be set"),
            11155111 => env::var("VITE_SEPOLIA_CONTRACT_ADDRESS").expect("CONTRACT_ADDRESS must be set"),
            1 => env::var("VITE_MAINNET_CONTRACT_ADDRESS").expect("CONTRACT_ADDRESS must be set"),
            10 => env::var("VITE_OPTIMISM_CONTRACT_ADDRESS").expect("CONTRACT_ADDRESS must be set"),
            _ => panic!("Invalid CURRENT_CHAIN_ID: {}", *CURRENT_CHAIN_ID),
        }
    };

    pub static ref RPC_URL: NodeOrRpcUrl = {
        let env_content = include_str!("../../../.env");
        from_read(Cursor::new(env_content)).expect("Failed to parse .env content");
        match *CURRENT_CHAIN_ID {
            31337 => NodeOrRpcUrl::RpcUrl(env::var("VITE_ANVIL_RPC_URL").expect("RPC_URL must be set")),
            11155111 => NodeOrRpcUrl::RpcUrl(env::var("VITE_SEPOLIA_RPC_URL").expect("RPC_URL must be set")),
            1 => NodeOrRpcUrl::RpcUrl(env::var("VITE_MAINNET_RPC_URL").expect("RPC_URL must be set")),
            10 => NodeOrRpcUrl::RpcUrl(env::var("VITE_OPTIMISM_RPC_URL").expect("RPC_URL must be set")),
            _ => panic!("Invalid CURRENT_CHAIN_ID: {}", *CURRENT_CHAIN_ID),
        }
    };

    pub static ref MIN_ETH_WAGER: U256 = {
        "100000000000000".parse().unwrap() // 0.0001 eth
    };
}

fn handle_http_request(
    state: &mut State,
    ws_channel_id: &mut Option<u32>,
    message: &Message,
    counter_caller: &mut Option<CounterCaller>,
) -> anyhow::Result<()> {
    let our_http_request = serde_json::from_slice::<http::HttpServerRequest>(message.body())?;
    match our_http_request {
        http::HttpServerRequest::WebSocketOpen { channel_id, .. } => {
            println!("got web socket open");
            *ws_channel_id = Some(channel_id);
            return Ok(());
        }
        http::HttpServerRequest::WebSocketClose { .. } => {
            *ws_channel_id = None;
            return Ok(());
        }
        http::HttpServerRequest::WebSocketPush {
            channel_id,
            message_type,
        } => {
            let Some(blob) = get_blob() else {
                return Ok(());
            };

            println!("got web socket push");
            if let None = counter_caller {
                println!("counter caller not instantied. please decrypt wallet first.");
                return Ok(());
            }
            let counter_caller = counter_caller.as_ref().unwrap();

            let ws_push = serde_json::from_slice::<WsPush>(&blob.bytes)?;
            match ws_push {
                WsPush::SetNumber(number) => {
                    let _ = counter_caller.set_number(number);
                    println!("Setting number to: {}", number);
                }
                WsPush::Increment => {
                    let _ = counter_caller.increment();
                    println!("Incremented");
                }
                WsPush::Number => {
                    let number = counter_caller.number()?;
                    println!("Got number: {}", number);
                    send_ws_push(
                        channel_id,
                        WsMessageType::Text,
                        LazyLoadBlob {
                            mime: Some("application/json".to_string()),
                            bytes: serde_json::to_vec(&WsUpdate::Number(number))?,
                        },
                    );
                }
                _ => {}
            }
            return Ok(());
        }
        _ => http::send_response(
            http::StatusCode::METHOD_NOT_ALLOWED,
            None,
            b"Method Not Allowed".to_vec(),
        ),
    }
    Ok(())
}

// used for eth contract testing
fn handle_terminal_message(
    state: &mut State,
    counter_caller: &mut Option<CounterCaller>,
    ws_channel_id: &mut Option<u32>,
    message: &Message,
) -> anyhow::Result<()> {
    println!("terminal message received");

    let action = match serde_json::from_slice::<Action>(&message.body()) {
        Ok(deserialized) => deserialized,
        Err(e) => {
            println!("Failed to deserialize message body: {:?}", e);
            return Ok(());
        }
    };

    match action {
        Action::EncryptWallet {
            private_key,
            password,
        } => {
            let private_key = match private_key {
                Some(private_key) => private_key,
                None => {
                    if let Some(PrivateKey::Decrypted(wallet)) =
                        state.wallets.get(&CURRENT_CHAIN_ID)
                    {
                        wallet.private_key.clone()
                    } else {
                        return Err(anyhow::anyhow!("Private key already encrypted."));
                    }
                }
            };
            let encrypted_wallet_data = encrypt_data(private_key.as_bytes(), password.as_str());
            state.wallets.insert(
                *CURRENT_CHAIN_ID,
                PrivateKey::Encrypted(encrypted_wallet_data),
            );
            state.save();

            if let Ok(parsed_wallet) = private_key.parse::<LocalWallet>() {
                println!(
                    "Loaded and encrypted wallet with address: {:?}",
                    parsed_wallet.address()
                );
            } else {
                println!("Failed to parse wallet key, try again.");
            }
            *counter_caller = None;
        }
        Action::DecryptWallet(password) => {
            if let Some(PrivateKey::Encrypted(encrypted_key)) = state.wallets.get(&CURRENT_CHAIN_ID)
            {
                match decrypt_data(&encrypted_key, &password) {
                    Ok(decrypted_key) => match String::from_utf8(decrypted_key)
                        .ok()
                        .and_then(|wd| wd.parse::<LocalWallet>().ok())
                    {
                        Some(parsed_wallet) => {
                            println!(
                                "Decrypted wallet with address: {:?}",
                                parsed_wallet.address()
                            );
                            let serializable_wallet = SerializableWallet::from(parsed_wallet);
                            state.wallets.insert(
                                *CURRENT_CHAIN_ID,
                                PrivateKey::Decrypted(serializable_wallet.clone()),
                            );
                            state.save();
                            if let Some(caller) = Caller::new(
                                *CURRENT_CHAIN_ID,
                                serializable_wallet.private_key.as_str(),
                            ) {
                                *counter_caller = Some(CounterCaller {
                                    caller: caller,
                                    contract_address: CONTRACT_ADDRESS.to_string(),
                                });
                            } else {
                                println!("Failed to create caller, try again.");
                                *counter_caller = None;
                            }
                        }
                        None => println!("Failed to parse wallet, try again."),
                    },
                    Err(_) => println!("Decryption failed, try again."),
                }
            } else {
                println!("to create wallet use EncryptWallet");
                println!("no wallet for chainid {}", *CURRENT_CHAIN_ID);
            }
        }
    }
    return Ok(());
}

fn handle_message(
    state: &mut State,
    counter_caller: &mut Option<CounterCaller>,
    ws_channel_id: &mut Option<u32>,
) -> anyhow::Result<()> {
    let message = await_message()?;

    if let "http_server:distro:sys" | "http_client:distro:sys" =
        message.source().process.to_string().as_str()
    {
        println!("HTTP request received.");
        return handle_http_request(state, ws_channel_id, &message, counter_caller);
    }
    if message.is_local(&message.source()) {
        println!("Local message received from: {:?}", message.source());

        if message.source().process.package_name == "terminal" {
            return handle_terminal_message(state, counter_caller, ws_channel_id, &message);
        }
    } else {
        println!("Message from invalid source: {:?}", message.source());
    }
    Ok(())
}

call_init!(init);
fn init(our: Address) {
    println!("{our}: eth template started");

    // UI INIT
    let mut ws_channel_id: Option<u32> = None;
    bind_ws_path("/", true, false).unwrap();
    let _ = http::serve_ui(&our, "ui", true, false, vec!["/"]);

    // STATE INIT
    let mut state = State::fetch().unwrap_or_else(|| State::new(&our));

    // ETH INIT
    // this doesnt work for now, need to manually add to eth providers
    let _ = Request::to(("our", "eth", "distro", "sys"))
        .body(
            serde_json::to_vec(&EthConfigAction::AddProvider(ProviderConfig {
                chain_id: *CURRENT_CHAIN_ID,
                trusted: true,
                provider: RPC_URL.clone(),
            }))
            .unwrap(),
        )
        .send();
    let mut counter_caller: Option<CounterCaller> = None;
    if let Some(PrivateKey::Decrypted(wallet)) = state.wallets.get(&CURRENT_CHAIN_ID) {
        counter_caller = Some(CounterCaller {
            caller: Caller::new(*CURRENT_CHAIN_ID, &wallet.private_key).unwrap(),
            contract_address: CONTRACT_ADDRESS.to_string(),
        });
    }

    loop {
        match handle_message(&mut state, &mut counter_caller, &mut ws_channel_id) {
            Ok(()) => {}
            Err(e) => {
                println!("error from somewhere: {:?}", e);
            }
        };
    }
}
