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

TODO - figure out why [this code](./eth_template/eth_template/src/lib.rs#L414-L423) isn't able to set eth providers programmatically

### Foundry Wallet
cast wallet import wallet-name --interactive
specific names
then when running ./deploy, you will be asked for a password
### Kinode Wallet
## Increment Contract

Specify the current chain id and it's rpc url in the `.env` file.
Then:
`./deploy.sh`

Copy the contract address from the output of the deploy script and paste it into the VITE_ANVIL_CONTRACT_ADDRESS field in the `.env` file.

### Wallet

- stores keys based on current chain id specified in .env setup

foudnry keys usage

#### Terminal

    - EncryptWallet {private_key: Option<String>, password: String}, // if none, will use decrypted wallet key
    `m our@eth_template:eth_template:astronaut.os '{"EncryptWallet": {"private_key": "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80", "password": "pass"}}'`
    - DecryptWallet(String),
    `m our@eth_template:eth_template:astronaut.os '{"DecryptWallet": "pass"}'`

### anvil and metamask

when using anvil from metamask, and the transactions stay pending, do the following:

- delete anvil network from metamask and re add it
- delete tx history tab in metamask (advanced settings)

- TODO
as im writing this, make sure to use get_logs_safely instead of get_logs in get_increment_logs. test that it works correctly.

### Actions

#### UI / WS

    - increment
    - set number
    - number

#### Terminal

    GetIncrementLogs(u64), // from block
    - `m our@eth_template:eth_template:astronaut.os '{"GetIncrementLogs": 0}'`
    ManyIncrements(u64),
    SubscribeIncrementLogs,
    UnsubscribeIncrementLogs,

## Getting Logs Safely

    GetUsdcLogs{from_block: u64, to_block: u64}// from_block, to_block
