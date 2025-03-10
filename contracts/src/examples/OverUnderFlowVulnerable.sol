// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract OverUnderFlowVulnerable {
    uint256 public balance;

    constructor() {
        balance = 1000;
    }

    function deposit(uint256 amount) public {
        unchecked {
            balance += amount;
        }
    }

    function withdraw(uint256 amount) public {
        unchecked {
            balance -= amount;
        }
    }
        
    // Fallback function to receive Ether
    receive() external payable {}
}
