// contracts/scripts/Deploy.s.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "forge-std/Script.sol";
import "../src/VerifierContract.sol";
import "../src/TargetContract.sol";

contract DeployScript is Script {
    function run() external {
        vm.startBroadcast();

        // Read deployment parameters from environment variables.
        // Foundry provides cheatcodes for reading env variables.
        address risc0Verifier = vm.envAddress("RISC0_VERIFIER_ADDRESS");
        bytes32 programSpecHash = vm.envBytes32("PROGRAM_SPEC_HASH");
        bytes32 contextStateHash = vm.envBytes32("CONTEXT_STATE_HASH");

        // Deploy the VerifierContract with the computed hashes.
        VerifierContract verifier = new VerifierContract(
            risc0Verifier,
            programSpecHash,
            contextStateHash
        );

        // Optionally deploy the TargetContract.
        TargetContract target = new TargetContract();

        vm.stopBroadcast();

        // Log the deployed addresses.
        console.log("VerifierContract deployed at:", address(verifier));
        console.log("TargetContract deployed at:", address(target));
    }
}
