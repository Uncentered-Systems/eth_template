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

#### Getting Logs Safely

To demonstrate getting a large amount of logs safely, we get logs from USDC contract on OP Mainnet.

In `.env`, change VITE_CURRENT_CHAIN_ID to 10 and run recompile the package.

`m our@eth_template:eth_template:astronaut.os '{"GetUsdcLogs": {"from_block": 123865000, "to_block": 123865806}}'`

## Code Explanation

### `sol-contracts`

`sol-contracts` contains all the usual foundry code for deployment, testing, etc., but also, the code for integration with the Kinode package is included.

`build.sh` copies the abi into the ui and into the rust backend, which they both use to interact with the contract.

`deploy.sh` is the script that will deploy the contract to the chain specified in .env.

### `eth_template`


#### Caller struct in `caller.rs`
generalizes struct for interacting with contract.
implements methods which are contract-agnostic, and will be used by contractcaller

#### ContractCaller struct in `contract_caller.rs`
takes caller struct and implements specific methods for interacting with each contract.

#### Getting Logs Safely Algorithm

### WebSocket Usage
