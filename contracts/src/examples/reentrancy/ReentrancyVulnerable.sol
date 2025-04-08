// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract ReentrancyVulnerable {
    mapping(address => uint256) public balances;

    function depositETH() public payable {
        balances[msg.sender] += msg.value;
    }

    function withdrawETH() public {
        uint256 balance = balances[msg.sender];

        // Send ETH 
        (bool success, ) = msg.sender.call{value: balance}("");
        require(success, "Withdraw failed");

        // Update Balance -- Vulnerable to reentrancy, the balance is updated after the ETH transfer
        balances[msg.sender] = 0;
    }
}
