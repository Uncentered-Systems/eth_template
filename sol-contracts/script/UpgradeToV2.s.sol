
// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script, console} from "forge-std/Script.sol";
import {CounterV2} from "../src/CounterV2.sol";
import {Upgrades, Options} from "openzeppelin-foundry-upgrades/Upgrades.sol";

contract UpgradeCounterToV2 is Script {
    CounterV2 public counter;
 
    function setUp() public {}

    function run(address proxy_address) public {
        vm.startBroadcast();

        Options memory opts;
        opts.referenceContract = "Counter.sol";

        console.log("MSG SENDER", msg.sender);

        Upgrades.upgradeProxy(
            proxy_address,
            "CounterV2.sol",
            "",
            opts
        );

        counter = new CounterV2();

        vm.stopBroadcast();
    }   
}
