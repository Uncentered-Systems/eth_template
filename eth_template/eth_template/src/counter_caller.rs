use crate::eth_utils::Caller;
use crate::CURRENT_CHAIN_ID;
use alloy_primitives::{FixedBytes, U256};
use alloy_sol_types::{sol, SolCall};
use kinode_process_lib::{
    println, eth::{Filter, BlockNumberOrTag, Address as EthAddress, Log}
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/* ABI import */
sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    #[derive(Debug, Deserialize, Serialize)]
    COUNTER,
    "abi/Counter.json"
);

pub type NumberIncrementedLog = Log<COUNTER::NumberIncremented>;

pub struct CounterCaller {
    pub caller: Caller,
    pub contract_address: String,
}

impl CounterCaller {
    pub fn set_number(&self, number: U256) -> anyhow::Result<FixedBytes<32>> {
        let call = COUNTER::setNumberCall { newNumber: number }.abi_encode();
        match self.caller.send_tx(
            call,
            &self.contract_address,
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
        self.caller.send_tx_with_nonce(nonce, call, &self.contract_address, 1500000, 10000000000, 300000000, U256::from(0), *CURRENT_CHAIN_ID)
    }

    pub fn increment(&self) -> anyhow::Result<(FixedBytes<32>, u64)> {
        let call = COUNTER::incrementCall {}.abi_encode();
        println!("here");
        match self.caller.send_tx(
            call,
            &self.contract_address,
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

    pub fn number(&self) -> anyhow::Result<U256> {
        let call: Vec<u8> = COUNTER::numberCall {}.abi_encode();
        match self.caller.tx_req(call, &self.contract_address) {
            Ok(bytes) => {
                let number = U256::from_be_slice(&bytes);
                Ok(number)
            }
            Err(e) => Err(anyhow::anyhow!("Error calling number: {:?}", e)),
        }
    }

    pub fn get_increment_logs(&self, from_block: u64) -> anyhow::Result<Vec<Log<COUNTER::NumberIncremented>>> {
        let filter: Filter = Filter::new()
        .address(EthAddress::from_str(&self.contract_address).unwrap())
        .from_block(from_block)
        .to_block(BlockNumberOrTag::Latest);

        let logs = self.caller.get_logs(&filter)?;
        let mut result = Vec::new();
        logs.iter().for_each(|log| {
            if let Ok(decoded) = log.log_decode::<COUNTER::NumberIncremented>() {
                result.push(decoded);
            }
        });
        Ok(result)
    }
}
