# Template for Kinode x Ethereum Interaction

## Dev Setup

[Install foundry.](https://book.getfoundry.sh/getting-started/installation)

Build contract and copy the abi so that Rust and JS can read it.

```bash
cd sol-contracts
./build.sh
```

Install eth_template package on your node.

```bash
cd eth_template
kit bs
```

Run a dev server.

```bash
cd eth_template/ui
npm run dev
```

Run anvil.

```bash
anvil
```

### .env

In `.env` you specify the addresses of the contracts, rpc urls, and the current chain id you want the app to work with.
The variables are prefixed with `VITE_` so they can also be used by the UI.

After modifying `.env`, to make the changes propagate,

1.  restart the dev server
2.  make a meaningless change in eth_template/src and run `kit bs`.

`lazy_static`s at the top of the `lib.rs` file are where .env file is being read on the backend.
Top of the `main.jsx` file is where .env is being read on the frontend.

### Eth Providers

Go to your node's home folder, and open the `.eth_providers` file.
Whichever chain you want to use, will need to have a rpc url set.
For anvil, add the following into the list:

```json
{
  "chain_id": 31337,
  "trusted": true,
  "provider": {
    "RpcUrl": "ws://localhost:8545"
  }
}
```

TODO - figure out why [this code](./eth_template/eth_template/src/lib.rs#L414-L423) isn't able to set eth providers programmatically.

### Foundry Wallet

Set up your foundry wallet.
In place of wallet-name, use `anvil`, `optimism`, `mainnet`, or `sepolia`, to insert the private key for each of these, respectively.
These names are hardcoded into the contract `deploy.sh` script.
When running `./deploy.sh`, you will be asked for the password you input here.

```bash
cast wallet import <wallet-name> --interactive
```

### Kinode Wallet

Depending on the current chain_id the process is compiled with (as specified in .env), the terminal commands shown below will store the key specifically for that chain_id.

To store the key encrypted in state, use:

`m our@eth_template:eth_template:astronaut.os '{"EncryptWallet": {"private_key": "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80", "password": "some-password"}}'`

To be able to interact with the contract, you need to decrypt the key.
Be careful, it will be stored in your kinode state unencrypted.

`m our@eth_template:eth_template:astronaut.os '{"DecryptWallet": "some-password"}'`

After you're done with using it, re-encrypt the key.

`m our@eth_template:eth_template:astronaut.os '{"EncryptWallet": {"password": "some-password"}}'`

### Metamask and Anvil

Add Anvil network to Metamask. Use `http://localhost:8545` as the RPC URL and `31337` as the chain ID.

Sometimes, the transactions from Metamask on Anvil will stay pending indefinitely.
In that case, do the following:

- Delete Anvil network from Metamask and re-add it.
- Clear activity tab data in Metamask (Settings -> Advanced).

## Counter Contract

Specify the current chain id and its rpc url in the `.env` file.
Then:
`./deploy.sh`

Copy the contract address from the output of the deploy script and paste it into the VITE_ANVIL_CONTRACT_ADDRESS field in the `.env` file.
[Recompile the process and restart the server.](#env)

### Interacting with Counter Contract

From the UI, you can interact with the counter contract in 2 ways.

#### 1. UI - BE - Chain

Send an action to the backend from the UI via WS, which will then make a call to the chain.

#### 2. Metamask - Chain

Make sure that your connected Metamask account is your Anvil account on the Anvil network.
Click "Connect Metamask".
Then you can talk to Anvil directly from Metamask.

### Other Actions

There are a few other actions for demo purposes which can be accessed from the terminal.

#### Get Increment Logs

Get all logs of events of type "NumberIncremented" and store them to local index. Starting from block 0. (This is fine if you are using anvil).

`m our@eth_template:eth_template:astronaut.os '{"GetIncrementLogs": 0}'`

Subscribe to logs of events of type "NumberIncremented" and store them to local index. After subscribing, when you make an increment, the index will be updated.

`m our@eth_template:eth_template:astronaut.os "SubscribeIncrementLogs"`

Unsubscribe from logs of events of type "NumberIncremented".

`m our@eth_template:eth_template:astronaut.os "UnsubscribeIncrementLogs"`

Make many increments; a convenience command for testing.

`m our@eth_template:eth_template:astronaut.os '{"ManyIncrements": 5}'`

#### Getting Usdc Logs Safely

To demonstrate getting a large amount of logs safely, we get logs from USDC contract on OP Mainnet.

In `.env`, change VITE_CURRENT_CHAIN_ID to 10 and run recompile the package.

`m our@eth_template:eth_template:astronaut.os '{"GetUsdcLogs": {"from_block": 123865000, "to_block": 123865806}}'`

## Code Explanation

### `sol-contracts`

`sol-contracts` contains all the usual foundry code for deployment, testing, etc., but also, the code for integration with the Kinode package is included.

`build.sh` copies the abi into the ui and into the rust backend, which they both use to interact with the contract.

`deploy.sh` is the script that will deploy the contract to the chain specified in .env.

### `eth_template`

#### `Caller` struct in `caller.rs`

A generalized struct containing methods for interacting with the chain.

Used by `ContractCaller` struct.

#### `ContractCaller` struct in `contract_caller.rs`

Contains `Caller` struct.

Implements methods for interacting with multiple contracts, using the primitives from the Caller struct.

To interact with each contract, it imports the contract ABI using `sol!` macro from alloy.

#### SubscribeIncrementLogs Example

`Filter` struct is used to specify which logs to subscribe to, see `subscribe_increment_logs` in `contract_caller.rs`.

Subscription updates are handled with `handle_eth_message` function.

#### `Filter` Usage

`Filter` is used whenever getting or subscribing to logs.

Example filter from `GetUsdcLogs` action:

```rust
    let address = EthAddress::from_str(
        &eth_caller.contract_addresses.get(&ContractName::Usdc).unwrap()
    ).unwrap();

    let sender_address = EthAddress::from_str("0xC8373EDFaD6d5C5f600b6b2507F78431C5271fF5").unwrap();
    let mut sender_topic_bytes = [0u8; 32];
    sender_topic_bytes[12..].copy_from_slice(&sender_address.to_vec());
    let sender_topic: FixedBytes<32> = FixedBytes::from_slice(&sender_topic_bytes);

    let filter: Filter = Filter::new()
        .address(address)
        .from_block(from_block)
        .to_block(to_block)
        .event("Transfer(address,address,uint256)")
        .topic1(sender_topic);
```

`address` specifies the address of the contract from which we are fetching logs.
`from_block` and `to_block` specify the desired range.

`event` specifies what type of event we are fetching, as defined in the ABI.
`topic1`, `topic2`, `topic3` would in this example refer to `address` (from), `address` (to), and `uint256` (value) in the event.
`topic1`, as shown in the code, is used to filter for events where `address` (from) is equal to `sender_address`.

All arguments in the filter are optional, but it is recommended to always use `address`, `from_block`, and `to_block`.


#### Getting Logs Safely

`get_logs_safely` functions allow for getting an arbitrary amount of logs.
If the chunk size is too large for any subset of the block range, they will retry with a halved chunk size, thus making getting logs safe.
It is recommended to use `get_logs_safely_binary_search()` for most cases.

`caller.get_logs_safely_binary_search()`

It approximates the largest amount that can be fetched at once by trial and error, and then recursively fetches logs in the requested range until the entire range has been fetched.

`caller.get_logs_safely_linear()`

It takes a chunk size, and then recursively fetches chunks of logs in the requested range until the entire range has been fetched.

To specify a range, use a `Filter`.
`get_logs_safely_linear` only supports a `Filter` which has:
- `from_block` as BlockNumberOrTag::Number
- `to_block` as BlockNumberOrTag::Number or BlockNumberOrTag::Latest

For an example, try [Getting Usdc Logs Safely](#getting-usdc-logs-safely).

An approximate benchmark demonstrated that cca 30 days of logs of all transfers on Usdc contract on Optimism Mainnet was fetched in 5.5 minutes.

### WebSocket Usage

Using "Get Number" from the UI as an example.

The chain of messages is as follows:
UI -ws-> BE --> chain --> BE -ws-> UI
