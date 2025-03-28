// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract MockRiscZeroVerifier {
    function verify(
        bytes calldata seal,
        bytes32 imageId,
        bytes32 journalDigest
    ) external view {
        // do nothing
    }
}
