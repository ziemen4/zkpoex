// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract OverUnderFlowVulnerable {
    uint256 public balance;

    constructor() {
        balance = 1000;
    }

    function deposit(uint256 amount) public {
        balance += amount; // Vulnerabilità: se 'amount' è troppo grande, può causare un overflow
    }

    function withdraw(uint256 amount) public {
        balance -= amount; // Vulnerabilità: se 'amount' è troppo grande, può causare un underflow
    }
}
