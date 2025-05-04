// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import {IRiscZeroVerifier} from "../lib/risc0-ethereum/contracts/src/IRiscZeroVerifier.sol";
import {Ownable} from "../lib/openzeppelin-contracts/contracts/access/Ownable.sol";

/// @notice Interface for the ImageID contract.
interface IImageID {
    function ZKPOEX_GUEST_ID() external view returns (bytes32);
}

contract VerifierContract is Ownable {
    /// @notice Image ID of the only zkVM binary to accept verification from.
    bytes32 public imageId;
    /// @notice Reward amount (in wei) for a valid exploit.
    uint256 public constant REWARD_IN_ETH = 1000;
    /// @notice Boolean indicating if contract is locked for exploit proof verification.
    bool public exploit_proof_verification_locked = false;

    IRiscZeroVerifier public immutable risc0_verifier_contract;

    // The stored hashes are computed as keccak256(abi.encodePacked(input))
    bytes32 public program_spec_hash;
    bytes32 public context_state_hash;

    event ExploitFound(address indexed beneficiary, address indexed verifier);

    constructor(
        address _risc0_verifier_contract,
        bytes32 _program_spec_hash,
        bytes32 _context_state_hash,
        address _image_id_contract
    ) Ownable(msg.sender) {
        risc0_verifier_contract = IRiscZeroVerifier(_risc0_verifier_contract);
        program_spec_hash = _program_spec_hash;
        context_state_hash = _context_state_hash;
        imageId = IImageID(_image_id_contract).ZKPOEX_GUEST_ID();
    }

    /// @notice Returns a boolean indicating if the contract is locked for exploit proof verification.
    /// @dev This function is used to check if the contract is locked.
    /// @return Boolean indicating if the contract is locked.
    function isLocked() external view returns (bool) {
        return exploit_proof_verification_locked;
    }

    /// @notice Locks the contract to prevent further exploit proof verification.
    /// @dev Only callable by the contract owner.
    /// @dev This function is used to lock the contract from further exploit proof verification.
    function lock() public onlyOwner {
        exploit_proof_verification_locked = true;
    }

    /// @notice Unlocks the contract to allow further exploit proof verification.
    /// @dev Only callable by the contract owner.
    /// @dev This function is used to unlock the contract for further exploit proof verification.
    function unlock() public onlyOwner {
        exploit_proof_verification_locked = false;
    }

    /// @notice Updates the target contract and the expected hashes.
    /// @dev Only callable by the contract owner.
    function updateVerifierFields(
        bytes32 _program_spec_hash,
        bytes32 _context_state_hash
    ) public onlyOwner {
        // TODO: Add some kind of delay to avoid owners from front-running provers.
        program_spec_hash = _program_spec_hash;
        context_state_hash = _context_state_hash;
    }

    /// @notice Verifies the public input and proof (seal) from the prover.
    ///         It calls the external risc0 verifier, then decodes and checks the public input.
    ///         If all checks pass, it emits an ExploitFound event and transfers the reward.
    function verify(
        address beneficiary,
        bytes calldata seal,
        bytes calldata journal
    ) public payable {
        // Check if contract is locked for exploit proof verification.
        require(
            !exploit_proof_verification_locked,
            "Contract is locked for exploit proof verification"
        );
        
        risc0_verifier_contract.verify(seal, imageId, sha256(journal));

        (
            bool exploit_found,
            bytes32 claimed_program_spec_hash,
            bytes32 claimed_context_state_hash
        ) = abi.decode(journal, (bool, bytes32, bytes32));

        // Check that an exploit was indeed found.
        require(exploit_found, "Exploit not found");

        // Validate that the provided hashes (after keccak256) match the stored values.
        // or equivalently, that the context state and program spec used to generate the proof
        // are the same as the ones stored in the contract.
        require(
            claimed_program_spec_hash == program_spec_hash,
            "Invalid program spec hash"
        );
        require(
            claimed_context_state_hash == context_state_hash,
            "Invalid context state hash"
        );

        // Emit event
        emit ExploitFound(beneficiary, address(this));
        require(payable(beneficiary).send(REWARD_IN_ETH), "Transfer failed");

        // Lock the contract to prevent further exploit proof verification until it is unlocked.
        exploit_proof_verification_locked = true;
    }

    receive() external payable {}
}
