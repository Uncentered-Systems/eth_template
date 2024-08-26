// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Test, console} from "forge-std/Test.sol";
import {Counter} from "../src/Counter.sol";
import {CounterV2} from "../src/CounterV2.sol";

contract CounterTest is Test {
    Counter public counter;
    CounterV2 public counterv2;

    function setUp() public {
        counter = new Counter();
        counterv2 = new CounterV2();
    }

    function testFuzz_Increment(uint256 x) public {
        x = bound(x, 1, type(uint256).max - 1);
        counter.setNumber(x);
        counter.increment();
        assertEq(counter.number(), x + 1);
        counterv2.setNumber(x);
        counterv2.increment();
        assertEq(counterv2.number(), x + 1);
    }

    function testFuzz_SetNumber(uint256 x) public {
        counter.setNumber(x);
        assertEq(counter.number(), x);
        counterv2.setNumber(x);
        assertEq(counterv2.number(), x);
    }

    function testFuzz_Decrement(uint256 x) public {
        x = bound(x, 1, type(uint256).max);
        counterv2.setNumber(x);
        counterv2.decrement();
        assertEq(counterv2.number(), x - 1);
    }
}
