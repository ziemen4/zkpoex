// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;


/*
TODO: Implement
- This contract should receive the public input and proof from the prover 
- It should send the proof to a Risc0 zkVM Verifier Contract and wait for the result, in case of failure revert the transaction
- In case of success, first it must verify that the public inputs are correct
- 1. Check that hash(program_spec) is equal to the claimed program_spec_hash in the public input
- 2. Check that hash(bytecode) is equal to the claimed bytecode_hash in the public input
- 3. Check that the public_key is equal to the claimed public_key in the public input
- 4. For any contracts that are not the target, also verify point 2 against those contracts (if any exist).
    * This contracts are contracts that are not the target contract, but are used in the program spec
    * i.e USDT balance > 0 or something like that
- If all checks pass, then we have two use cases:
- * A new exploit found, that is a condition C_i of the program spec is violated
    - This case occurs if the public input has a boolean flag for "exploit_found" set to true
    - In this case, check that the hash(program_spec_exploit_category_mapping) is equal to the 
      claimed program_spec_exploit_category_mapping_hash in the public input
    - If the check passes, then we know that the category sent in the public input is correct
    - Therefore, we find the reward associated with that category and send it to the msg.sender (prover)
        * (TODO) Here we should verify that msg.sender is actually the prover or someone could front run the reward

- * A new condition found, that is a condition C_j not in the program spec should be added, and in that case is violated
    - This case occurs if the public input has a boolean flag for "condition_found" set to true
    - (TODO) R&D on how we could prove that this new condition maps to a category. Here we may need a decentralized third party
             that votes on this somehow while the encrypted calldata is still encrypted
    - For now, we default to the lowest category in case the condition is deemed valid.
    - Here we must store the proper values so that when the protocol wants it can update this contract with the new program_spec
      including the new program_spec and the new program_spec_exploit_category_mapping
    - At that point, when updating the program spec, in case there are any pending new conditions, we will first verify this.
      In case the program_spec is updating to use said condition then a reward for the lowest category will be given to the prover
    - To avoid DoS and protocol having to go through with unnecesasry conditions, a collateral must be put in place for the prover
      to be able to submit a new condition. This collateral will be returned in case the condition is valid and the protocol accepts it
      or after a certain time period has passed and the protocol has not accepted it
*/

import { IRiscZeroVerifier } from "risc0/IRiscZeroVerifier.sol";
import { ImageID } from "./ImageID.sol"; // auto-generated contract after running `cargo build`.

contract VerifierContract {

    address public target_contract;
    IRiscZeroVerifier public immutable risc0_verifier_contract;
    address public owner;

    address[] public exploit_categories;
    mapping(address => uint256) public exploit_category_rewards;

    bytes32 public program_spec_hash;
    bytes32 public bytecode_hash;
    bytes32 public public_key;
    bytes32 public program_spec_exploit_category_mapping_hash;
    bytes32[] public contract_bytecode_hashes;

    event ExploitFound(address indexed prover, address indexed exploit_category, uint256 reward);
    event ConditionFound(address indexed prover, uint256 reward);

    constructor(address _target_contract, address _risc0_verifier_contract) {
        target_contract = _target_contract;
        risc0_verifier_contract = IRiscZeroVerifier(_risc0_verifier_contract);
        owner = msg.sender;
    }

    function verify(
        bytes memory public_input,
        bytes calldata seal
    ) public {
        bytes memory journal = abi.encode(public_input);
        bool result = risc0_verifier_contract.verify(seal, imageId, sha256(journal));
        require(result, "Verification failed");

        (
            bool exploit_found, 
            bool condition_found, 
            bytes32 program_spec_hash, 
            bytes32 bytecode_hash, 
            address public_key, 
            bytes32 program_spec_exploit_category_mapping_hash,
            bytes32[] memory contract_bytecode_hashes
        ) = abi.decode(public_input, (bool, bool, bytes32, bytes32, address, bytes32, bytes32[]));

        // Check that hash(program_spec) is equal to the claimed program_spec_hash in the public input
        require(keccak256(abi.encodePacked(program_spec_hash)) == program_spec_hash, "Invalid program spec hash");

        // Check that hash(bytecode) is equal to the claimed bytecode_hash in the public input
        require(keccak256(abi.encodePacked(bytecode_hash)) == bytecode_hash, "Invalid bytecode hash");

        // Check that the public_key is equal to the claimed public_key in the public input
        require(public_key == public_key, "Invalid public key");

        // For any contracts that are not the target, also verify point 2 against those contracts (if any exist).
        // This contracts are contracts that are not the target contract, but are used in the program spec
        // i.e USDT balance > 0 or something like that
        for (uint256 i = 0; i < contract_bytecode_hashes.length; i++) {
            require(keccak256(abi.encodePacked(contract_bytecode_hashes[i])) == contract_bytecode_hashes[i], "Invalid contract bytecode hash");
        }
    }
}