use crate::caller::Caller;
use crate::CURRENT_CHAIN_ID;
use alloy_primitives::{FixedBytes, U256};
use alloy_sol_types::{sol, SolCall};
use kinode_process_lib::{
    eth::{Address as EthAddress, BlockNumberOrTag, Filter, Log},
    println,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

/* ABI import */
sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    #[derive(Debug, Deserialize, Serialize)]
    USDC,
    "abi/Usdc.json"
);

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    #[derive(Debug, Deserialize, Serialize)]
    COUNTER,
    "abi/CounterV2.json"
);

#[derive(Eq, Hash, PartialEq)]
pub enum ContractName {
    Usdc,
    Counter,
}

pub struct ContractCaller {
    pub caller: Caller,
    pub contract_addresses: HashMap<ContractName, String>,
}

impl ContractCaller {
    // usdc methods

    // counter methods
    pub fn set_number(&self, number: U256) -> anyhow::Result<FixedBytes<32>> {
        let call = COUNTER::setNumberCall { newNumber: number }.abi_encode();
        match self.caller.send_tx(
            call,
            &self
                .contract_addresses
                .get(&ContractName::Counter)
                .unwrap_or(&"".to_string()),
            1500000,
            10000000000,
            300000000,
            U256::from(0),
            *CURRENT_CHAIN_ID,
        ) {
            Ok((tx_hash, _nonce)) => Ok(tx_hash),
            Err(e) => Err(anyhow::anyhow!("Error setting number: {:?}", e)),
        }
    }

    pub fn increment_with_nonce(&self, nonce: u64) -> anyhow::Result<(FixedBytes<32>, u64)> {
        let call = COUNTER::incrementCall {}.abi_encode();
        self.caller.send_tx_with_nonce(
            nonce,
            call,
            &self
                .contract_addresses
                .get(&ContractName::Counter)
                .unwrap_or(&"".to_string()),
            1500000,
            10000000000,
            300000000,
            U256::from(0),
            *CURRENT_CHAIN_ID,
        )
    }

    pub fn increment(&self) -> anyhow::Result<(FixedBytes<32>, u64)> {
        let call = COUNTER::incrementCall {}.abi_encode();
        println!("here");
        match self.caller.send_tx(
            call,
            &self
                .contract_addresses
                .get(&ContractName::Counter)
                .unwrap_or(&"".to_string()),
            1500000,
            10000000000,
            300000000,
            U256::from(0),
            *CURRENT_CHAIN_ID,
        ) {
            Ok((tx_hash, nonce)) => {
                println!("tx_hash: {:?}", tx_hash);
                Ok((tx_hash, nonce))
            }
            Err(e) => {
                println!("Error incrementing counter: {:?}", e);
                Err(anyhow::anyhow!("Error incrementing counter: {:?}", e))
            }
        }
    }

    // only exists in CounterV2
    pub fn decrement(&self) -> anyhow::Result<(FixedBytes<32>, u64)> {
        let call = COUNTER::decrementCall {}.abi_encode();
        println!("decrementing");
        match self.caller.send_tx(
            call,
            &self
                .contract_addresses
                .get(&ContractName::Counter)
                .unwrap_or(&"".to_string()),
            1500000,
            10000000000,
            300000000,
            U256::from(0),
            *CURRENT_CHAIN_ID,
        ) {
            Ok((tx_hash, nonce)) => {
                println!("tx_hash: {:?}", tx_hash);
                Ok((tx_hash, nonce))
            }
            Err(e) => {
                println!("Error decrementing counter: {:?}", e);
                Err(anyhow::anyhow!("Error decrementing counter: {:?}", e))
            }
        }
    }


    pub fn number(&self) -> anyhow::Result<U256> {
        let call: Vec<u8> = COUNTER::numberCall {}.abi_encode();
        match self.caller.tx_req(
            call,
            &self
                .contract_addresses
                .get(&ContractName::Counter)
                .unwrap_or(&"".to_string()),
        ) {
            Ok(bytes) => {
                let number = U256::from_be_slice(&bytes);
                Ok(number)
            }
            Err(e) => Err(anyhow::anyhow!("Error calling number: {:?}", e)),
        }
    }

    pub fn get_increment_logs(&self, from_block: u64) -> anyhow::Result<HashMap<u64, U256>> {
        let filter: Filter = Filter::new()
            .address(
                EthAddress::from_str(
                    &self
                        .contract_addresses
                        .get(&ContractName::Counter)
                        .unwrap_or(&"".to_string()),
                )
                .unwrap(),
            )
            .from_block(from_block)
            .to_block(BlockNumberOrTag::Latest)
            .event("NumberIncremented(uint256)");

        let logs = self.caller.get_logs_safely_binary_search(&filter)?;
        let mut index: HashMap<u64, U256> = HashMap::new();
        logs.iter().for_each(|log| {
            if let Ok(inc) = log.log_decode::<COUNTER::NumberIncremented>() {
                let COUNTER::NumberIncremented { newNumber } = inc.inner.data;
                index.insert(log.block_timestamp.unwrap_or_default(), newNumber);
            }
        });
        Ok(index)
    }

    pub fn subscribe_increment_logs(&self, contract_name: ContractName) -> anyhow::Result<()> {
        let filter: Filter = Filter::new()
            .address(
                EthAddress::from_str(&self.contract_addresses.get(&contract_name).unwrap())
                    .unwrap(),
            )
            .event("NumberIncremented(uint256)");

        self.caller.subscribe_logs(&filter)
    }

    pub fn unsubscribe_increment_logs(&self, contract_name: ContractName) -> anyhow::Result<()> {
        let filter: Filter = Filter::new()
            .address(
                EthAddress::from_str(&self.contract_addresses.get(&contract_name).unwrap())
                    .unwrap(),
            )
            .event("NumberIncremented(uint256)");

        self.caller.unsubscribe_logs(&filter)
    }
}
