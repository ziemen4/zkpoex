// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Test.sol";
import "../src/VerifierContract.sol";
import "./mocks/MockRiscZeroVerifier.sol";

contract VerifierContractTest is Test {
    VerifierContract public verifierContract;
    MockRiscZeroVerifier public mockVerifier;

    // Constants for the expected verifier fields.
    bytes32 constant TEST_PROGRAM_SPEC =
        0x1111111111111111111111111111111111111111111111111111111111111111;
    bytes32 constant TEST_BYTECODE =
        0x2222222222222222222222222222222222222222222222222222222222222222;
    bytes32 constant TEST_CONTEXT_DATA =
        0x3333333333333333333333333333333333333333333333333333333333333333;
    address constant TARGET = address(0xdead);

    function setUp() public {
        // Deploy the mock RiscZero verifier.
        mockVerifier = new MockRiscZeroVerifier();
        // Deploy the VerifierContract with initial parameters.
        verifierContract = new VerifierContract(
            TARGET,
            address(mockVerifier),
            TEST_PROGRAM_SPEC,
            TEST_BYTECODE,
            TEST_CONTEXT_DATA
        );
    }

    // The event as declared in VerifierContract.
    event ExploitFound(
        address indexed prover,
        address indexed exploit_category,
        uint256 reward
    );

    function test_exploit() public {
        // Fund the contract with the reward so that the send call succeeds.
        vm.deal(address(verifierContract), verifierContract.REWARD_IN_ETH());

        // Prepare a dummy seal (the mock verifier's verify does nothing).
        bytes memory dummySeal = hex"00";

        // Build the public input.
        // Note: The public input encodes:
        //   (bool exploit_found, bytes32 _program_spec_hash, bytes32 _bytecode_hash, bytes32 _context_data_hash, address _prover_address)
        bool exploit_found = true;
        address prover = address(0x123);
        bytes memory publicInput = abi.encode(
            exploit_found,
            TEST_PROGRAM_SPEC,
            TEST_BYTECODE,
            TEST_CONTEXT_DATA,
            prover
        );

        // Expect the ExploitFound event.
        vm.expectEmit(true, true, false, true);
        emit ExploitFound(
            prover,
            address(verifierContract),
            verifierContract.REWARD_IN_ETH()
        );

        // Call verify; since the mock doesn't revert, all checks should pass.
        verifierContract.verify{value: 0}(publicInput, dummySeal);

        // Confirm that the prover received the reward.
        assertEq(
            prover.balance,
            verifierContract.REWARD_IN_ETH(),
            "Prover should receive reward"
        );
    }

    function test_updateVerifierFields() public {
        // Define new values for the verifier fields.
        bytes32 newProgramSpec = 0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa;
        bytes32 newBytecode = 0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb;
        bytes32 newContextData = 0xcccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc;
        address newTarget = address(0xbeef);

        // Update the verifier fields (only the owner can do this).
        verifierContract.updateVerifierFields(
            newTarget,
            newProgramSpec,
            newBytecode,
            newContextData
        );

        // Verify that the fields have been updated correctly.
        assertEq(
            verifierContract.target_contract(),
            newTarget,
            "Target contract should update"
        );
        assertEq(
            verifierContract.program_spec_hash(),
            keccak256(abi.encodePacked(newProgramSpec)),
            "Program spec hash should update"
        );
        assertEq(
            verifierContract.bytecode_hash(),
            keccak256(abi.encodePacked(newBytecode)),
            "Bytecode hash should update"
        );
        assertEq(
            verifierContract.context_data_hash(),
            keccak256(abi.encodePacked(newContextData)),
            "Context data hash should update"
        );
    }
}
