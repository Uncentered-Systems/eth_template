use crate::counter_caller::CounterCaller;
use alloy_primitives::U256;
use serde::{Serialize, Deserialize};
use kinode_process_lib::{Address, get_state, set_state};
use std::collections::HashMap;
use alloy_signer::{k256::ecdsa::SigningKey, Wallet, LocalWallet, Signer};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum WsPush {
    Increment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    EncryptWallet {private_key: Option<String>, password: String}, // if none, will use decrypted wallet key
    DecryptWallet(String),
    SetNumber(U256),
    Increment,
    Number,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrivateKey {
    Encrypted(Vec<u8>),
    Decrypted(SerializableWallet),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SerializableWallet {
    pub address: String,
    pub private_key: String,
}

impl From<LocalWallet> for SerializableWallet {
    fn from(wallet: LocalWallet) -> Self {
        SerializableWallet {
            address: wallet.address().to_string(),
            private_key: hex::encode(wallet.to_bytes()),
        }
    }
}

impl SerializableWallet {
    pub fn new() -> Self {
        SerializableWallet {
            address: "".to_string(),
            private_key: "".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub our: Address,
    pub wallets: HashMap<u64, PrivateKey>, //chain id to wallet
}

impl State {
    pub fn new(our: &Address) -> Self {
        State {
            our: our.clone(),
            wallets: HashMap::new(),
        }
    }
    pub fn fetch() -> Option<State> {
        if let Some(state_bytes) = get_state() {
            bincode::deserialize(&state_bytes).ok()
        } else {
            None
        }
    }
    pub fn save(&self) {
        let serialized_state = bincode::serialize(self).expect("Failed to serialize state");
        set_state(&serialized_state);
    }
}
