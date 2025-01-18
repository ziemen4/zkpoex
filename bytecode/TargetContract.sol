// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract TargetContract {
    // Function to exploit the contract
    function exploit(bool _exploit) public {
        if (_exploit) {
            // Burn all balance by sending it to address(0)
            payable(address(0)).transfer(address(this).balance);
        }
    }

    // Function that supposedly has no exploit
    function supposedly_no_exploit(uint256 _number) public {
        uint256 specificNumber = 123456789; // Replace with the specific number
        if (_number == specificNumber) {
            // Burn all balance by sending it to address(0)
            payable(address(0)).transfer(address(this).balance);
        }
    }

    // Fallback function to receive Ether
    receive() external payable {}
}