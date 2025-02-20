// SPDX-License-Identifier: MIT

contract MockRiscZeroVerifier {
    function verify(bytes calldata seal, bytes32 imageId, bytes32 journalDigest) external view {
        // do nothing
    }
}