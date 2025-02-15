// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

/*
- This contract should receive the public input and proof from the prover 
- It should send the proof to a Risc0 zkVM Verifier Contract and wait for the result, in case of failure revert the transaction
- In case of success, first it must verify that the public inputs are correct
- 1. Check that the exploit_found boolean is true
- 1. Check that hash(program_spec) is equal to the claimed program_spec_hash in the public input
- 2. Check that hash(bytecode) is equal to the claimed bytecode_hash in the public input
- 3. Check that hash(context_data) is equal to the claimed context_data_hash in the public input

- If all checks pass, then:
- * A new exploit found, that is a condition C_i of the program spec is violated
    - Find the reward associated to finding an exploit for this method and give it to the prover
        * (TODO) Here we should verify that msg.sender is actually the prover or someone could front run the reward
*/

import { IRiscZeroVerifier } from "risc0/IRiscZeroVerifier.sol";
import { ImageID } from "./ImageID.sol"; // auto-generated contract after running `cargo build`.

contract VerifierContract {
    /// @notice Image ID of the only zkVM binary to accept verification from.
    ///         The image ID is similar to the address of a smart contract.
    ///         It uniquely represents the logic of that guest program,
    ///         ensuring that only proofs generated from a pre-defined guest program
    ///         (in this case, checking if a number is even) are considered valid.
    bytes32 public constant imageId = ImageID.ZKPOEX_GUEST_ID;

    address public owner;
    address public target_contract;
    IRiscZeroVerifier public immutable risc0_verifier_contract;

    bytes32 public program_spec_hash;
    bytes32 public bytecode_hash;
    bytes32 public context_data_hash;

    event ExploitFound(address indexed prover, address indexed exploit_category, uint256 reward);

    constructor(address _target_contract, address _risc0_verifier_contract) {
        target_contract = _target_contract;
        risc0_verifier_contract = IRiscZeroVerifier(_risc0_verifier_contract);
        owner = msg.sender;
    }

    function verify(
        bytes memory public_input,
        bytes calldata seal
    ) public payable{
        bytes memory journal = abi.encode(public_input);
        risc0_verifier_contract.verify(seal, imageId, sha256(journal));

        (
            bool exploit_found, 
            bytes32 _program_spec_hash, 
            bytes32 _bytecode_hash, 
            bytes32 _context_data_hash
        ) = abi.decode(public_input, (bool, bytes32, bytes32, bytes32));

        // 1. Check that exploit_found is true
        require(exploit_found, "Exploit not found");

        // 2. Check that hash(program_spec) is equal to the claimed program_spec_hash in the public input
        require(keccak256(abi.encodePacked(_program_spec_hash)) == program_spec_hash, "Invalid program spec hash");

        // 3. Check that hash(bytecode) is equal to the claimed bytecode_hash in the public input
        require(keccak256(abi.encodePacked(_bytecode_hash)) == bytecode_hash, "Invalid bytecode hash");

        // 4. Check that hash(context_data) is equal to the claimed context_data_hash in the public input
        require(keccak256(abi.encodePacked(_context_data_hash)) == context_data_hash, "Invalid context data hash");

        // If all checks pass, reward the prover
        emit ExploitFound(msg.sender, target_contract, 1000);
        
        // Transfer REWARD_IN_ETH to msg.sender
        require(payable(msg.sender).send(1000), "Transfer failed");
    }
}