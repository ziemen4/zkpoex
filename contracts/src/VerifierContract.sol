// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import { IRiscZeroVerifier } from "../lib/risc0-ethereum/contracts/src/IRiscZeroVerifier.sol";

/// @notice Interface for the ImageID contract. 
interface IImageID {
    function ZKPOEX_GUEST_ID() external view returns (bytes32);
}

contract VerifierContract {
    /// @notice Image ID of the only zkVM binary to accept verification from.
    bytes32 public imageId;
    /// @notice Reward amount (in wei) for a valid exploit.
    uint256 public constant REWARD_IN_ETH = 1000;

    address public owner;
    IRiscZeroVerifier public immutable risc0_verifier_contract;

    // The stored hashes are computed as keccak256(abi.encodePacked(input))
    bytes32 public program_spec_hash;
    bytes32 public context_state_hash;

    event ExploitFound(address indexed prover, address indexed exploit_category, uint256 reward);

    constructor(
        address _risc0_verifier_contract,
        bytes32 _program_spec_hash,
        bytes32 _context_state_hash,
        address _image_id_contract
    ) {
        risc0_verifier_contract = IRiscZeroVerifier(_risc0_verifier_contract);
        owner = msg.sender;
        program_spec_hash = _program_spec_hash;
        context_state_hash = _context_state_hash; 
        imageId = IImageID(_image_id_contract).ZKPOEX_GUEST_ID();
    }

    /// @notice Updates the target contract and the expected hashes.
    /// @dev Only callable by the contract owner.
    function updateVerifierFields(
        bytes32 _program_spec_hash,
        bytes32 _context_state_hash
    ) external {
        // TODO: Add some kind of delay to avoid owners from front-running provers.
        require(msg.sender == owner, "Only owner");
        program_spec_hash = _program_spec_hash;
        context_state_hash = _context_state_hash;
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
            bytes32 claimed_program_spec_hash, 
            bytes32 claimed_context_state_hash, 
            address prover_address
        ) = abi.decode(public_input, (bool, bytes32, bytes32, address));

        // Check that an exploit was indeed found.
        require(exploit_found, "Exploit not found");

        // Validate that the provided hashes (after keccak256) match the stored values.
        require(claimed_program_spec_hash == program_spec_hash, "Invalid program spec hash");
        require(claimed_context_state_hash == context_state_hash, "Invalid context state hash");

        // Emit event and transfer reward to the prover.
        emit ExploitFound(prover_address, address(this), REWARD_IN_ETH);
        require(payable(prover_address).send(REWARD_IN_ETH), "Transfer failed");
    }
}
