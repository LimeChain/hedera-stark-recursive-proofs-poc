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

import {RiscZeroCheats} from "lib/risc0-ethereum/contracts/src/test/RiscZeroCheats.sol";
import {console2} from "forge-std/console2.sol";
import {Test} from "forge-std/Test.sol";
import {IRiscZeroVerifier} from "lib/risc0-ethereum/contracts/src/IRiscZeroVerifier.sol";
import {CommitmentVerification} from "../contracts/CommitmentVerification.sol";
import {Elf} from "./Elf.sol"; // auto-generated contract after running `cargo build`.

contract CommitmentVerificationTest is RiscZeroCheats, Test {
    CommitmentVerification public commitmentVerification;

    bytes journal;
    bytes seal;

    function setUp() public {
        IRiscZeroVerifier verifier = deployRiscZeroVerifier();
        commitmentVerification = new CommitmentVerification(verifier);

        string[] memory imageRunnerInput = new string[](8);
        uint256 i = 0;
        imageRunnerInput[i++] = "cargo";
        imageRunnerInput[i++] = "run";
        imageRunnerInput[i++] = "--release";
        imageRunnerInput[i++] = "-F";
        imageRunnerInput[i++] = "cuda";
        imageRunnerInput[i++] = "--";
        imageRunnerInput[i++] = "--pubkey0";
        imageRunnerInput[i++] = "8cdd23bfd1e38ddb4ae9539f5947847bf56d3f06404d6f385758a4faa7443507e96b00da1a13d84b2fdf1a65958201352c69896103fc76a6b3c967808ad099ae";

        bytes memory data = vm.ffi(imageRunnerInput);

        console2.logBytes(data);

        (journal, seal) =
            abi.decode(data, (bytes, bytes));
    }

    function test_proverWorkflow() public view {
        commitmentVerification.verify(journal, seal);
    }
}
