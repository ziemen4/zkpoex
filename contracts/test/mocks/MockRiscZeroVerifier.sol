// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
contract MockRiscZeroVerifier {
    function verify(
        bytes calldata seal,
        bytes32 imageId,
        bytes32 journalDigest
    ) external view {
        // This is a mock function, so we don't need to implement any logic here.
        // In a real contract, this function would verify the zk proof and
        // return a boolean indicating success or failure.
        // For this mock, we just return without doing anything.
        // In a real contract, you would typically return a boolean indicating
        // whether the verification was successful or not.

        return;
    }
}
