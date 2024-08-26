#!/bin/bash

if [ -f ../.env ]; then
  source ../.env
else
  echo ".env file not found!"
  exit 1
fi

if [ $# -ne 1 ]; then
  echo "Usage: $0 <script_name>"
  exit 1
fi

script_name=$1

case $VITE_CURRENT_CHAIN_ID in
    1)
        echo "Deploying to Mainnet"
        account="mainnet"
        rpc_url=$VITE_MAINNET_RPC_URL
        proxy_address=$VITE_MAINNET_CONTRACT_ADDRESS
        ;;
    11155111)
        echo "Deploying to Sepolia"
        account="sepolia"
        rpc_url=$VITE_SEPOLIA_RPC_URL
        proxy_address=$VITE_SEPOLIA_CONTRACT_ADDRESS
        ;;
    10)
        echo "Deploying to Optimism"
        account="optimism"
        rpc_url=$VITE_OPTIMISM_RPC_URL
        proxy_address=$VITE_OPTIMISM_CONTRACT_ADDRESS
        ;;
    31337)
        echo "Deploying to Anvil"
        account="anvil"
        rpc_url=$VITE_ANVIL_RPC_URL
        proxy_address=$VITE_ANVIL_CONTRACT_ADDRESS
        ;;
    *)
        echo "Unknown chain ID: $VITE_CURRENT_CHAIN_ID"
        exit 1
        ;;
esac

read -s -p "Enter keystore password: " password
our_address=$(sshpass -p "$password" cast wallet address --account anvil --password $password)

case $script_name in
    "Deploy.s.sol")
        sshpass -p "$password" forge script --rpc-url $rpc_url script/$script_name --broadcast --account $account --sender $our_address
        ;;
    "UpgradeToV2.s.sol")
        sshpass -p "$password" forge script --rpc-url $rpc_url script/$script_name --broadcast --account $account --sender $our_address --sig "run(address)" $proxy_address
        ;;
    *)
        ;;
esac

echo "$script_name script done"
