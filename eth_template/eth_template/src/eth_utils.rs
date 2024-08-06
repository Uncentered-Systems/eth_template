use alloy::{
    consensus::{SignableTransaction, TxEip1559, TxEnvelope},
    network::eip2718::Encodable2718,
    network::TxSignerSync,
    primitives::TxKind,
    rpc::types::eth::TransactionRequest,
    signers::local::PrivateKeySigner,
};
use alloy_primitives::{FixedBytes, U256};
use kinode_process_lib::{
    eth::{
        Address as EthAddress, BlockNumberOrTag, EthError, Filter, FilterBlockOption, Log, Provider,
    },
    println,
};
use std::str::FromStr;

pub struct Caller {
    pub provider: Provider,
    pub signer: PrivateKeySigner,
}

impl Caller {
    pub fn new(chain_id: u64, wallet_addr: &str) -> Option<Self> {
        // get wallet address
        let wallet_address;
        if let Ok(wallet) = PrivateKeySigner::from_str(wallet_addr) {
            wallet_address = wallet;
        } else {
            return None;
        }
        Some(Self {
            provider: Provider::new(chain_id, 5),
            signer: wallet_address,
        })
    }

    pub fn tx_req(
        &self,
        call: Vec<u8>,
        contract_address: &str,
    ) -> Result<alloy_primitives::Bytes, EthError> {
        let tx_req = TransactionRequest::default();
        let to = match EthAddress::from_str(contract_address) {
            Ok(to) => to,
            Err(e) => return Err(EthError::MalformedRequest),
        };
        let tx = tx_req.to(to).input(call.into());
        self.provider.call(tx, None)
    }

    pub fn send_tx_with_nonce(
        &self,
        nonce: u64,
        call: Vec<u8>,
        contract_address: &str,
        gas_limit: u128,
        max_fee_per_gas: u128,
        max_priority_fee_per_gas: u128,
        value: U256,
        chain_id: u64,
    ) -> anyhow::Result<(FixedBytes<32>, u64)> {
        let to;
        if let Ok(address) = EthAddress::from_str(contract_address) {
            to = address;
        } else {
            return Err(anyhow::anyhow!("Invalid contract address"));
        }

        let mut tx = TxEip1559 {
            chain_id: chain_id,
            nonce: nonce,
            to: TxKind::Call(to),
            gas_limit: gas_limit,
            max_fee_per_gas: max_fee_per_gas,
            max_priority_fee_per_gas: max_priority_fee_per_gas,
            input: call.into(),
            value: value,
            ..Default::default()
        };

        let sig = self.signer.sign_transaction_sync(&mut tx)?;
        let signed = TxEnvelope::from(tx.into_signed(sig));
        let mut buf = vec![];
        signed.encode_2718(&mut buf);

        let result = self.provider.send_raw_transaction(buf.into());
        match result {
            Ok(tx_hash) => Ok((tx_hash, nonce)),
            Err(e) => Err(anyhow::anyhow!("Error sending transaction: {:?}", e)),
        }
    }
    pub fn send_tx(
        &self,
        call: Vec<u8>,
        contract_address: &str,
        gas_limit: u128,
        max_fee_per_gas: u128,
        max_priority_fee_per_gas: u128,
        value: U256,
        chain_id: u64,
    ) -> anyhow::Result<(FixedBytes<32>, u64)> {
        // get nonce
        println!("here1");
        let mut nonce = 0;
        let tx_count = self
            .provider
            .get_transaction_count(self.signer.address(), None);
        println!("tx_count: {:?}", tx_count);
        if let Ok(tx_count) = tx_count {
            nonce = tx_count.to::<u64>();
            println!("nonce: {:?}", nonce);
        } else {
            println!("tx_count: {:?}", tx_count);
            return Err(anyhow::anyhow!("Error getting transaction count"));
        }

        // get contract address
        let to;
        if let Ok(address) = EthAddress::from_str(contract_address) {
            to = address;
        } else {
            return Err(anyhow::anyhow!("Invalid contract address"));
        }

        let mut tx = TxEip1559 {
            chain_id: chain_id,
            nonce: nonce,
            to: TxKind::Call(to),
            gas_limit: gas_limit,
            max_fee_per_gas: max_fee_per_gas,
            max_priority_fee_per_gas: max_priority_fee_per_gas,
            input: call.into(),
            value: value,
            ..Default::default()
        };

        let sig = self.signer.sign_transaction_sync(&mut tx).unwrap();
        let signed = TxEnvelope::from(tx.into_signed(sig));
        let mut buf = vec![];
        signed.encode_2718(&mut buf);

        let result = self.provider.send_raw_transaction(buf.into());
        match result {
            Ok(tx_hash) => Ok((tx_hash, nonce)),
            Err(e) => Err(anyhow::anyhow!("Error sending transaction: {:?}", e)),
        }
    }

    pub fn get_latest_block(&self) -> anyhow::Result<u64> {
        match self.provider.get_block_number() {
            Ok(block) => Ok(block),
            Err(e) => Err(anyhow::anyhow!("Error getting latest block: {:?}", e)),
        }
    }

    pub fn get_logs(&self, filter: &Filter) -> anyhow::Result<Vec<Log>> {
        match self.provider.get_logs(&filter) {
            Ok(logs) => Ok(logs),
            Err(e) => {
                println!("FAILED to fetch logs: {:?}", e);

                Ok(Vec::new())
            }
        }
    }

    
    /*
    outside loop - once success received, divide by correct number of times
    inner loop - keep trying until first success
     */
    pub fn get_logs_safely(&self, filter: &Filter) -> anyhow::Result<Vec<Log>> {
        let latest_block = self.get_latest_block()?;
        let mut starting_block: u64;
        if let FilterBlockOption::Range {
            from_block,
            to_block,
        } = filter.block_option
        {
            if let BlockNumberOrTag::Number(from_block) = from_block.unwrap() {
                starting_block = from_block;
            } else {
                return Err(anyhow::anyhow!("Invalid from_block"));
            }
        } else {
            return Err(anyhow::anyhow!("Invalid from_block"));
        }
        /* 
        length = latest - successful
        to_block = latest
        while starting block < from_block {
            to_block = latest - length
            from_block = to_block - length
            get logs, add to vec
        }
        from-block = starting
        to-block = previous_to_block
        get logs, add to vec

        */

        // TODO nova logika, appenda dok ne dode do kraja lol
        // nema fiksni length, nego svaki unutarnji loop ce izbacivat nesto drukcije. i onda samo appendat 

        println!("INITIAL LOOP START");

        let mut logs: Vec<Log> = Vec::new();
        // buld up vector here (of all logs)
        let (initial_logs, successful_from_block) =
        self.get_logs_safely_inner_loop(&filter, latest_block)?;
        logs.splice(0..0, initial_logs.into_iter()); //prepends logs
        println!("got: {} - {}", successful_from_block, latest_block);
        let length = latest_block - successful_from_block + 1;

        let mut to_block = latest_block;
        let mut from_block = successful_from_block + 1;
        println!("INITIAL LOOP END");
        while starting_block < from_block {
            
            to_block -= length;
            from_block = to_block - length; 
            
            println!("ask: {} - {}", from_block, to_block);
            let mut new_filter = filter.clone();
            new_filter.block_option = FilterBlockOption::Range {
                from_block: Some(BlockNumberOrTag::Number(from_block)),
                to_block: Some(BlockNumberOrTag::Number(to_block)),
            };

            let (new_logs, new_successful_from_block) =
                self.get_logs_safely_inner_loop(&new_filter, latest_block)?;
            println!("got: {} - {}", new_successful_from_block, to_block);
            logs.splice(0..0, new_logs.into_iter()); //prepends logs
        }

        // Final fetch for the remaining range
        let mut final_filter = filter.clone();
        final_filter.block_option = FilterBlockOption::Range {
            from_block: Some(BlockNumberOrTag::Number(starting_block)),
            to_block: Some(BlockNumberOrTag::Number(to_block)),
        };

        let (final_logs, _) = self.get_logs_safely_inner_loop(&final_filter, latest_block)?;
        logs.splice(0..0, final_logs.into_iter()); //prepends logs

        Ok(logs)
    }

    // implements safety net such that large logs can be queried as well
    // NOTE: works correctly only when from_block is BlockNumberOrTag::Number, and to_block is BlockNumberOrTag::Latest
    pub fn get_logs_safely_inner_loop(
        &self,
        filter: &Filter,
        latest_block: u64,
    ) -> anyhow::Result<(Vec<Log>, u64)> {
        match filter.block_option {
            FilterBlockOption::Range {
                from_block,
                to_block,
            } => match self.provider.get_logs(&filter) {
                Ok(logs) => {
                    // println!("success from block: {:?}", from_block);
                    if let BlockNumberOrTag::Number(from_block) = from_block.unwrap() {
                        return Ok((logs, from_block));
                    } else {
                        return Err(anyhow::anyhow!("Invalid from_block"));
                    }
                }
                Err(e) => {
                    // println!("error fetching logs: {:?}", e);
                    // println!("when trying block: {:?}", from_block);
                    if let BlockNumberOrTag::Number(from_block) = from_block.unwrap() {
                        let filter = filter.clone().from_block((from_block + latest_block) / 2);
                        self.get_logs_safely_inner_loop(&filter, latest_block)
                    } else {
                        return Err(anyhow::anyhow!("Invalid from_block"));
                    }
                }
            },
            FilterBlockOption::AtBlockHash(block_hash) => match self.provider.get_logs(&filter) {
                Ok(logs) => Ok((logs, latest_block)),
                Err(e) => {
                    println!("FAILED to fetch logs: {:?}", e);
                    Err(anyhow::anyhow!("Error fetching logs"))
                }
            },
        }
    }

    pub fn subscribe_logs(&self, sub_id: u64, filter: &Filter) -> anyhow::Result<()> {
        match self.provider.subscribe(sub_id, filter.clone()) {
            Ok(_) => Ok(()),
            Err(e) => {
                println!("failed to subscribe: {:?}", e);
                Err(anyhow::anyhow!("Error subscribing!"))
            }
        }
    }

    pub fn unsubscribe_logs(&self, sub_id: u64) -> anyhow::Result<()> {
        match self.provider.unsubscribe(sub_id) {
            Ok(_) => Ok(()),
            Err(e) => {
                println!("failed to unsubscribe: {:?}", e);
                Err(anyhow::anyhow!("Error unsubscribing!"))
            }
        }
    }
}
