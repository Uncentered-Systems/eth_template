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
use std::hash::{DefaultHasher, Hash, Hasher};
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
        let mut nonce = 0;
        let tx_count = self
            .provider
            .get_transaction_count(self.signer.address(), None);
        // println!("tx_count: {:?}", tx_count);
        if let Ok(tx_count) = tx_count {
            nonce = tx_count.to::<u64>();
            // println!("nonce: {:?}", nonce);
        } else {
            // println!("tx_count: {:?}", tx_count);
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
    outside loop - stack inner loop until all desired data is received
    inner loop - keep trying until first success
     */
    pub fn get_logs_safely(&self, filter: &Filter) -> anyhow::Result<Vec<Log>> {
        let latest_block = self.get_latest_block()?;
        let starting_block: u64;

        // annoying unwrapping
        if let FilterBlockOption::Range {
            from_block: Some(BlockNumberOrTag::Number(from_block)),
            ..
        } = filter.block_option
        {
            starting_block = from_block;
        } else {
            return Err(anyhow::anyhow!("Please use FilterBlockOption::Range"));
        }

        println!("INITIAL LOOP START");

        let mut logs: Vec<Log> = Vec::new();
        // buld up vector here (of all logs)
        let mut flag = true;
        let mut length;
        let mut filter = filter.clone();
        while flag {
            println!(
                "ask: {:?} - {:?}",
                filter.block_option.get_from_block(),
                filter.block_option.get_to_block()
            );
            let (new_logs, successful_from_block, mut successful_to_block) =
                self.get_logs_safely_inner_loop(&filter, None)?;
            logs.splice(0..0, new_logs.into_iter()); //prepends logs
            
            if let BlockNumberOrTag::Latest = successful_to_block {
                successful_to_block = BlockNumberOrTag::Number(latest_block);
            }
            if let (
                BlockNumberOrTag::Number(successful_from_block),
                BlockNumberOrTag::Number(successful_to_block),
            ) = (successful_from_block, successful_to_block)
            {
                println!(
                    "got: {:?} - {:?}",
                    successful_from_block, successful_to_block
                );
                if successful_from_block <= starting_block {
                    flag = false;
                }
                length = successful_to_block - successful_from_block + 1;
                filter.block_option = FilterBlockOption::Range {
                    from_block: Some(BlockNumberOrTag::Number(successful_from_block - length)),
                    to_block: Some(BlockNumberOrTag::Number(successful_from_block - 1)),
                };
            } else {
                return Err(anyhow::anyhow!("Invalid inner loop output"));
            }
        }

        println!("LOOP END");
        Ok(logs)
    }

    // if cannot fetch "from - to", tries fetching "(from - to)/2 - to"  recursively until it finds a range that works
    //
    // implements safety net such that large logs can be queried as well
    // NOTE: works correctly only when
    // - FilterBlockOption::Range
    // - from_block is BlockNumberOrTag::Number
    // - to_block is BlockNumberOrTag::Latest or BlockNumberOrTag::Number
    // returns (logs, from_block, to_block)
    pub fn get_logs_safely_inner_loop(
        &self,
        filter: &Filter,
        latest_block: Option<u64>,
    ) -> anyhow::Result<(Vec<Log>, BlockNumberOrTag, BlockNumberOrTag)> {
        // annoying unwrapping

        let from: u64;
        let to: BlockNumberOrTag;

        if let FilterBlockOption::Range {
            from_block: Some(BlockNumberOrTag::Number(from_block)),
            to_block: Some(to_block),
        } = filter.block_option
        {
            from = from_block;
            to = to_block;
        } else {
            return Err(anyhow::anyhow!("Please use FilterBlockOption::Range"));
        }

        match self.provider.get_logs(&filter) {
            Ok(logs) => {
                println!("success from - to: {:?} - {:?}", from, to);
                return Ok((logs, BlockNumberOrTag::Number(from), to));
            }
            Err(_e) => {
                let latest_block = latest_block.unwrap_or(self.get_latest_block()?);
                match to {
                    BlockNumberOrTag::Latest => {
                        let filter = filter.clone().from_block((from + latest_block) / 2);
                        println!("trying from - to: {:?} - {:?}", from, filter.get_to_block());
                        self.get_logs_safely_inner_loop(&filter, Some(latest_block))
                    }
                    BlockNumberOrTag::Number(to) => {
                        let filter = filter.clone().from_block((from + to) / 2);
                        println!("trying from - to: {:?} - {:?}", from, filter.get_to_block());
                        self.get_logs_safely_inner_loop(&filter, Some(latest_block))
                    }
                    _ => {
                        return Err(anyhow::anyhow!("Please use BlockNumberOrTag::Number or BlockNumberOrTag::Latest for to_block"));
                    }
                }
            }
        }
    }

    // sub_id = hashed filter
    pub fn subscribe_logs(&self, filter: &Filter) -> anyhow::Result<()> {
        let mut hasher: DefaultHasher = DefaultHasher::new();
        filter.hash(&mut hasher);
        match self.provider.subscribe(hasher.finish(), filter.clone()) {
            Ok(_) => Ok(()),
            Err(e) => {
                println!("failed to subscribe: {:?}", e);
                Err(anyhow::anyhow!("Error subscribing!"))
            }
        }
    }

    // sub_id = hashed filter
    pub fn unsubscribe_logs(&self, filter: &Filter) -> anyhow::Result<()> {
        let mut hasher: DefaultHasher = DefaultHasher::new();
        filter.hash(&mut hasher);
        match self.provider.unsubscribe(hasher.finish()) {
            Ok(_) => Ok(()),
            Err(e) => {
                println!("failed to unsubscribe: {:?}", e);
                Err(anyhow::anyhow!("Error unsubscribing!"))
            }
        }
    }
}
