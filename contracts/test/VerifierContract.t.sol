// Copyright 2024 RISC Zero, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.20;

import {RiscZeroCheats} from "risc0/test/RiscZeroCheats.sol";
import {console2} from "forge-std/console2.sol";
import {Test} from "forge-std/Test.sol";
import {IRiscZeroVerifier} from "risc0/IRiscZeroVerifier.sol";
import {VerifierContract} from "../src/VerifierContract.sol";
import {Elf} from "./Elf.sol"; // auto-generated contract after running `cargo build`.

contract VerifierContractTest is RiscZeroCheats, Test {
    VerifierContract public verifierContract;

    function setUp() public {
        IRiscZeroVerifier verifier = deployRiscZeroVerifier();
        verifierContract = new VerifierContract(address(0x388C818CA8B9251b393131C08a736A67ccB19297), address(verifier));
    }

    function test_exploit() public {
        (bytes memory journal, bytes memory seal) = prove(
            Elf.ZKPOEX_GUEST_PATH,
            // TODO: Fix by somehow decoding correctly on the guest (or have a guest only for test-purposes)
            abi.encode(
                "16112c6c0000000000000000000000000000000000000000000000000000000000000001",
                "",
                "",
                "",
                "",
                ""
            )
        );

        verifierContract.verify(journal, seal);
    }
}