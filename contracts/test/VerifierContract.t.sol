// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Test.sol";
import "../src/VerifierContract.sol";
import "./mocks/MockRiscZeroVerifier.sol";
import "../src/ImageID.sol";

contract VerifierContractTest is Test {
    VerifierContract public verifierContract;
    MockRiscZeroVerifier public mockVerifier;
    address public imageID;

    // Constants for the expected verifier fields.
    bytes32 constant TEST_PROGRAM_SPEC =
        0x1111111111111111111111111111111111111111111111111111111111111111;
    bytes32 constant TEST_CONTEXT_STATE =
        0x2222222222222222222222222222222222222222222222222222222222222222;

    function setUp() public {
        bytes memory imageIDBytecode = type(ImageID).runtimeCode;
        address imageIDAddr = address(0x123456);
        vm.etch(imageIDAddr, imageIDBytecode);

        // Deploy the mock verifier contract.
        mockVerifier = new MockRiscZeroVerifier();
        address mockVerifierAddress = address(mockVerifier);

        verifierContract = new VerifierContract(
            mockVerifierAddress,
            TEST_PROGRAM_SPEC,
            TEST_CONTEXT_STATE,
            imageIDAddr
        );
    }


    // The event as declared in VerifierContract.
    event ExploitFound(address indexed beneficiary, address indexed verifier);

    function test_exploit() public {
        // Fund the contract with the reward so that the send call succeeds.
        vm.deal(address(verifierContract), verifierContract.REWARD_IN_ETH());

        // Beneficiary address for the reward.
        address beneficiary = address(0x456);

        // Prepare a dummy seal (the mock verifier's verify does nothing).
        bytes memory dummySeal = hex"00";

        // Build the public input.
        // Note: The public input encodes:
        //   (bool exploit_found, bytes32 _program_spec_hash, bytes32 _bytecode_hash, bytes32 _context_data_hash, address _prover_address)
        bool exploit_found = true;
        bytes memory publicInput = abi.encode(
            exploit_found,
            TEST_PROGRAM_SPEC,
            TEST_CONTEXT_STATE
        );

        // Expect the ExploitFound event.
        vm.expectEmit(true, true, false, false);
        emit ExploitFound(
            beneficiary,
            address(verifierContract)
        );

        // Call verify; since the mock doesn't revert, all checks should pass.
        verifierContract.verify(beneficiary, dummySeal, publicInput);

        // Confirm that the prover received the reward.
        assertEq(
            beneficiary.balance,
            verifierContract.REWARD_IN_ETH(),
            "Prover should receive reward"
        );
    }

    function test_updateVerifierFields() public {
        // Define new values for the verifier fields.
        bytes32 newProgramSpec = 0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa;
        bytes32 newContextData = 0xcccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc;

        // Update the verifier fields (only the owner can do this).
        verifierContract.updateVerifierFields(
            newProgramSpec,
            newContextData
        );

        // Verify that the fields have been updated correctly.
        assertEq(
            verifierContract.program_spec_hash(),
            newProgramSpec,
            "Program spec hash should update"
        );
        assertEq(
            verifierContract.context_state_hash(),
            newContextData,
            "Context data hash should update"
        );
    }
}
