use chrono::naive::NaiveDateDaysIterator;
use dotenvy::from_read;
use lazy_static::lazy_static;
use std::env;
use std::io::Cursor;
use std::time::Instant;

use crate::contract_caller::COUNTER::new;
use crate::contract_caller::{COUNTER, USDC};
use kinode_process_lib::http::{bind_ws_path, send_ws_push, WsMessageType};
use kinode_process_lib::{
    await_message, call_init,
    eth::{
        Address as EthAddress, BlockNumberOrTag, EthConfigAction, EthSubResult, Filter,
        NodeOrRpcUrl, ProviderConfig, SubscriptionResult,
    },
    get_blob,
    http::{self},
    println, Address, LazyLoadBlob, Message, Request, Response,
};
mod caller;
mod encryption;
use caller::Caller;
mod contract_caller;
use alloy_primitives::{keccak256, FixedBytes, Signature, B256, U256};
use alloy_signer::{LocalWallet, Signer};
use contract_caller::{ContractCaller, ContractName};
mod types;
use std::collections::HashMap;
use std::str::FromStr;
use types::{Action, PrivateKey, State, Wallet, WsPush, WsUpdate};

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

    pub static ref COUNTER_CONTRACT_ADDRESS: String = {
        let env_content = include_str!("../../../.env");
        from_read(Cursor::new(env_content)).expect("Failed to parse .env content");
        let contract_address = match *CURRENT_CHAIN_ID {
            31337 => env::var("VITE_ANVIL_CONTRACT_ADDRESS").expect("CONTRACT_ADDRESS must be set"),
            11155111 => env::var("VITE_SEPOLIA_CONTRACT_ADDRESS").expect("CONTRACT_ADDRESS must be set"),
            1 => env::var("VITE_MAINNET_CONTRACT_ADDRESS").expect("CONTRACT_ADDRESS must be set"),
            10 => env::var("VITE_OPTIMISM_CONTRACT_ADDRESS").expect("CONTRACT_ADDRESS must be set"),
            _ => panic!("Invalid CURRENT_CHAIN_ID: {}", *CURRENT_CHAIN_ID),
        };
        println!("NEW contract address: {}", contract_address);
        contract_address
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

    pub static ref USDC_CONTRACT_ADDRESS: String = {
        let env_content = include_str!("../../../.env");
        from_read(Cursor::new(env_content)).expect("Failed to parse .env content");
        let contract_address = match *CURRENT_CHAIN_ID {
            10 => env::var("VITE_OPTIMISM_USDC_CONTRACT_ADDRESS").expect("USDC CONTRACT_ADDRESS must be set"),
            _ => "".to_string(),
        };
        println!("NEW USDC contract address: {}", contract_address);
        contract_address
    };
}

fn initialize_addresses() -> HashMap<ContractName, String> {
    let mut map = HashMap::new();
    map.insert(ContractName::Counter, COUNTER_CONTRACT_ADDRESS.to_string());
    map.insert(ContractName::Usdc, USDC_CONTRACT_ADDRESS.to_string());
    map
}

fn handle_http_request(
    state: &mut State,
    ws_channel_id: &mut Option<u32>,
    message: &Message,
    eth_caller: &mut Option<ContractCaller>,
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
            if let None = eth_caller {
                println!("eth caller not instantied. please decrypt wallet first.");
                return Ok(());
            }
            let eth_caller = eth_caller.as_ref().unwrap();

            let ws_push = serde_json::from_slice::<WsPush>(&blob.bytes)?;
            match ws_push {
                WsPush::SetNumber(number) => {
                    let _ = eth_caller.set_number(number);
                    println!("Setting number to: {}", number);
                }
                WsPush::Increment => {
                    let _ = eth_caller.increment();
                    println!("Incremented");
                }
                WsPush::Number => {
                    let number = eth_caller.number()?;
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
    eth_caller: &mut Option<ContractCaller>,
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
            *eth_caller = None;
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
                            let wallet = Wallet::from(parsed_wallet);
                            state
                                .wallets
                                .insert(*CURRENT_CHAIN_ID, PrivateKey::Decrypted(wallet.clone()));
                            state.save();
                            if let Some(caller) =
                                Caller::new(*CURRENT_CHAIN_ID, wallet.private_key.as_str())
                            {
                                *eth_caller = Some(ContractCaller {
                                    caller: caller,
                                    contract_addresses: initialize_addresses(),
                                });
                            } else {
                                println!("Failed to create caller, try again.");
                                *eth_caller = None;
                            }
                        }
                        None => println!("Failed to parse wallet, try again."),
                    },
                    Err(_) => println!("Decryption failed, try again."),
                }
            } else {
                println!(
                    "either wallet already decrypted or no wallet created yet for chainid {}",
                    *CURRENT_CHAIN_ID
                );
            }
        }
        Action::ManyIncrements(num) => {
            if let None = eth_caller {
                println!("eth caller not instantied. please decrypt wallet first.");
                return Ok(());
            }
            let eth_caller = eth_caller.as_ref().unwrap();
            let result = eth_caller.increment()?;
            let mut nonce = result.1;
            for i in 1..num.try_into().unwrap() {
                let result = eth_caller.increment_with_nonce(nonce + 1)?;
                nonce = result.1;
            }
        }
        Action::GetIncrementLogs(from_block) => {
            if let None = eth_caller {
                println!("eth caller not instantied. please decrypt wallet first.");
                return Ok(());
            }
            let eth_caller = eth_caller.as_ref().unwrap();

            state.increment_log_index = eth_caller.get_increment_logs(from_block)?;

            // overwrites index from scratch
            // from_block usually supposed to be 0, for dev reasons its a var
            state.save();
            println!("state: {:#?}", state.increment_log_index);
        }
        Action::SubscribeIncrementLogs => {
            if let None = eth_caller {
                println!("eth caller not instantied. please decrypt wallet first.");
                return Ok(());
            }
            let eth_caller = eth_caller.as_ref().unwrap();

            eth_caller.subscribe_increment_logs(ContractName::Counter)?;
        }
        Action::UnsubscribeIncrementLogs => {
            if let None = eth_caller {
                println!("eth caller not instantied. please decrypt wallet first.");
                return Ok(());
            }
            let eth_caller = eth_caller.as_ref().unwrap();

            eth_caller.unsubscribe_increment_logs(ContractName::Counter)?;
        }
        Action::GetUsdcLogs {
            from_block,
            to_block,
        } => {
            if let None = eth_caller {
                println!("eth caller not instantied. please decrypt wallet first.");
                return Ok(());
            }
            let eth_caller: &ContractCaller = eth_caller.as_ref().unwrap();
            let address = EthAddress::from_str(
                &eth_caller
                    .contract_addresses
                    .get(&ContractName::Usdc)
                    .unwrap(),
            )
            .unwrap();

            let sender_address =
                EthAddress::from_str("0xC8373EDFaD6d5C5f600b6b2507F78431C5271fF5").unwrap();
            let mut sender_topic_bytes = [0u8; 32];
            sender_topic_bytes[12..].copy_from_slice(&sender_address.to_vec());
            let sender_topic: FixedBytes<32> = FixedBytes::from_slice(&sender_topic_bytes);

            let filter: Filter = Filter::new()
                .address(address)
                .from_block(from_block)
                .to_block(to_block)
                .event("Transfer(address,address,uint256)")
                .topic1(sender_topic);

            let start = Instant::now();
            let logs = eth_caller.caller.get_logs_safely_binary_search(&filter)?;
            let duration = start.elapsed();
            println!("Time elapsed: {:?}", duration);

            logs.iter().for_each(|log| {
                // println!("log: {:#?}", log);
                if let Ok(transfer) = log.log_decode::<USDC::Transfer>() {
                    println!("{:#?}", transfer.inner.data);
                    // let USDC::Transfer { .. } = inc.inner.data;
                }
            });
        }
    }
    return Ok(());
}

fn handle_eth_message(
    state: &mut State,
    eth_caller: &mut Option<ContractCaller>,
    message: &Message,
) -> anyhow::Result<()> {
    let eth_result = match serde_json::from_slice::<EthSubResult>(&message.body()) {
        Ok(deserialized) => deserialized,
        Err(e) => {
            println!("Failed to deserialize message body: {:?}", e);
            return Ok(());
        }
    };
    // println!("eth result: {:#?}", eth_result);
    if let Err(e) = eth_result {
        println!("error receiving eth result: {:#?}", e);
    } else {
        match eth_result.unwrap().result {
            SubscriptionResult::Log(log) => {
                // println!("log: {:#?}", log);
                if let Ok(decoded) = log.log_decode::<COUNTER::NumberIncremented>() {
                    state.increment_log_index.insert(
                        log.block_timestamp.unwrap_or_default(),
                        decoded.inner.data.newNumber,
                    );
                    state.save();
                    println!("index len: {:#?}", state.increment_log_index.len());
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn handle_message(
    state: &mut State,
    eth_caller: &mut Option<ContractCaller>,
    ws_channel_id: &mut Option<u32>,
) -> anyhow::Result<()> {
    let message = await_message()?;

    if let "http_server:distro:sys" | "http_client:distro:sys" =
        message.source().process.to_string().as_str()
    {
        println!("HTTP request received.");
        return handle_http_request(state, ws_channel_id, &message, eth_caller);
    }
    if let "eth:distro:sys" = message.source().process.to_string().as_str() {
        println!("ETH message received.");
        return handle_eth_message(state, eth_caller, &message);
    }
    if message.is_local(&message.source()) {
        println!("Local message received from: {:?}", message.source());

        if message.source().process.package_name == "terminal" {
            return handle_terminal_message(state, eth_caller, ws_channel_id, &message);
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
    let mut eth_caller: Option<ContractCaller> = None;
    if let Some(PrivateKey::Decrypted(wallet)) = state.wallets.get(&CURRENT_CHAIN_ID) {
        eth_caller = Some(ContractCaller {
            caller: Caller::new(*CURRENT_CHAIN_ID, &wallet.private_key).unwrap(),
            contract_addresses: initialize_addresses(),
        });
    }

    loop {
        match handle_message(&mut state, &mut eth_caller, &mut ws_channel_id) {
            Ok(()) => {}
            Err(e) => {
                println!("error from somewhere: {:?}", e);
            }
        };
    }
}
