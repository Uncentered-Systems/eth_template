// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "openzeppelin-contracts-upgradeable/contracts/proxy/utils/Initializable.sol";
import "openzeppelin-contracts-upgradeable/contracts/access/OwnableUpgradeable.sol";
import "openzeppelin-contracts-upgradeable/contracts/proxy/utils/UUPSUpgradeable.sol";

/// @custom:oz-upgrades-from Counter
contract CounterV2  is Initializable, OwnableUpgradeable, UUPSUpgradeable {
    uint256 public number;

    event NumberIncremented(uint256 newNumber);
    event NumberDecremented(uint256 newNumber);
    event NumberSet(uint256 newNumber);

    /// @custom:oz-upgrades-unsafe-allow constructor
    constructor() {
        _disableInitializers();
    }

    // called only when contract deployed initially, 
    // not meant to be called when contract is upgraded
    function initialize(address initialOwner) initializer public {
        __Ownable_init(initialOwner);
        __UUPSUpgradeable_init();
    }

    function _authorizeUpgrade(address newImplementation)
        internal
        onlyOwner
        override
    {}

    function setNumber(uint256 newNumber) public {
        number = newNumber;
        emit NumberSet(newNumber);
    }

    function increment() public {
        number++;
        emit NumberIncremented(number);
    }

    function decrement() public {
        number--;
        emit NumberDecremented(number);
    }
}
