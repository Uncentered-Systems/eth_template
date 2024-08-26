
// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script, console} from "forge-std/Script.sol";
import {Counter} from "../src/Counter.sol";
import {Upgrades} from "openzeppelin-foundry-upgrades/Upgrades.sol";

contract DeployCounter is Script {
    Counter public counter;

    function setUp() public {}

    function run() public {
        vm.startBroadcast();

        console.log("MSG SENDER", msg.sender);

        // ovo bi trebalo automatski deployat
        address proxy = Upgrades.deployUUPSProxy(
            "Counter.sol",
            abi.encodeCall(Counter.initialize, (msg.sender))
        );
        console.log("Proxy address: ", proxy);

        counter = new Counter();

        vm.stopBroadcast();
    }   
}
