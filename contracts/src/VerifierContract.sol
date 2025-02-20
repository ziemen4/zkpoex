// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import { IRiscZeroVerifier } from "risc0/IRiscZeroVerifier.sol";
import { ImageID } from "./ImageID.sol";

contract VerifierContract {
    /// @notice Image ID of the only zkVM binary to accept verification from.
    bytes32 public constant imageId = ImageID.ZKPOEX_GUEST_ID;
    /// @notice Reward amount (in wei) for a valid exploit.
    uint256 public constant REWARD_IN_ETH = 1000;

    address public owner;
    IRiscZeroVerifier public immutable risc0_verifier_contract;

    // The stored hashes are computed as keccak256(abi.encodePacked(input))
    bytes32 public program_spec_hash;
    bytes32 public bytecode_hash;
    bytes32 public context_data_hash;

    event ExploitFound(address indexed prover, address indexed exploit_category, uint256 reward);

    constructor(
        address _risc0_verifier_contract,
        bytes32 _program_spec_hash,
        bytes32 _bytecode_hash,
        bytes32 _context_data_hash
    ) {
        risc0_verifier_contract = IRiscZeroVerifier(_risc0_verifier_contract);
        owner = msg.sender;
        program_spec_hash = _program_spec_hash;
        bytecode_hash = _bytecode_hash; 
        context_data_hash = _context_data_hash;
    }

    /// @notice Updates the target contract and the expected hashes.
    /// @dev Only callable by the contract owner.
    function updateVerifierFields(
        bytes32 _program_spec_hash,
        bytes32 _bytecode_hash,
        bytes32 _context_data_hash
    ) external {
        // TODO: Add some kind of delay to avoid owners from front-running provers.
        require(msg.sender == owner, "Only owner");
        program_spec_hash = keccak256(abi.encodePacked(_program_spec_hash));
        bytecode_hash = keccak256(abi.encodePacked(_bytecode_hash));
        context_data_hash = keccak256(abi.encodePacked(_context_data_hash));
    }

    /// @notice Verifies the public input and proof (seal) from the prover.
    ///         It calls the external risc0 verifier, then decodes and checks the public input.
    ///         If all checks pass, it emits an ExploitFound event and transfers the reward.
    function verify(
        bytes memory public_input,
        bytes calldata seal
    ) public payable {
        // Compute the journal from public_input and pass its hash to the verifier.
        bytes memory journal = abi.encode(public_input);
        risc0_verifier_contract.verify(seal, imageId, sha256(journal));

        (
            bool exploit_found, 
            bytes32 _program_spec_hash, 
            bytes32 _bytecode_hash, 
            bytes32 _context_data_hash,
            address _prover_address
        ) = abi.decode(public_input, (bool, bytes32, bytes32, bytes32, address));

        // Check that an exploit was indeed found.
        require(exploit_found, "Exploit not found");

        // Validate that the provided hashes (after keccak256) match the stored values.
        require(keccak256(abi.encodePacked(_program_spec_hash)) == program_spec_hash, "Invalid program spec hash");
        require(keccak256(abi.encodePacked(_bytecode_hash)) == bytecode_hash, "Invalid bytecode hash");
        require(keccak256(abi.encodePacked(_context_data_hash)) == context_data_hash, "Invalid context data hash");

        // Emit event and transfer reward to the prover.
        emit ExploitFound(_prover_address, address(this), REWARD_IN_ETH);
        require(payable(_prover_address).send(REWARD_IN_ETH), "Transfer failed");
    }
}
