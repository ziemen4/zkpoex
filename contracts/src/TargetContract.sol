// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import { IERC20 } from "../lib/openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";

contract TargetContract {
    address public constant CONTEXT_ERC20_CONTRACT_ADDRESS = address(0xE4C2000000000000000000000000000000000000);

    // Function to exploit the contract
    function exploit(bool _exploit) public {
        if (_exploit) {
            // Burn all balance by sending it to address(0)
            payable(address(0)).transfer(address(this).balance);
        }
    }

    // Function to exploit the contract with ERC20
    function exploit_erc20(bool _exploit) public {
        if (_exploit) {
            // Burn all balance by sending it to address(0)
            IERC20 token = IERC20(CONTEXT_ERC20_CONTRACT_ADDRESS);
            require(token.transfer(address(0x0000000000000000000000000000000000000001), token.balanceOf(address(this))), "Transfer failed");
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