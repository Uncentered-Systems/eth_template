#!/bin/bash

forge build
# Extract the 'abi' field from the source JSON and write it to the target file
cp out/Counter.sol/Counter.json ../eth_template/eth_template/abi/Counter.json
cp out/Counter.sol/Counter.json ../eth_template/ui/src/abi/Counter.json
echo "ABI updated successfully."
