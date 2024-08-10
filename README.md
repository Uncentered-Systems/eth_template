# Template for Kinode x Ethereum Interaction

## Dev Setup
### foundry/anvil setup
./build.sh
./deploy.sh
### eth providers
    - rpc for each chain id
### .env
- when changing env, restart dev server, make a meaningless change in eth_template/src and run kit bs
### Wallet
- stores keys based on current chain id specified in .env setup
#### Terminal
    - EncryptWallet {private_key: Option<String>, password: String}, // if none, will use decrypted wallet key
    `m our@eth_template:eth_template:astronaut.os '{"EncryptWallet": {"private_key": "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80", "password": "pass"}}'`
    - DecryptWallet(String),
    `m our@eth_template:eth_template:astronaut.os '{"DecryptWallet": "pass"}'`

### anvil and metamask
when using anvil from metamask, and the transactions stay pending, do the following:
- delete anvil network from metamask and re add it
- delete tx history tab in metamask (advanced settings)

## Increment Contract
- as im writing this, make sure to use get_logs_safely instead of get_logs in get_increment_logs. test that it works correctly.

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
