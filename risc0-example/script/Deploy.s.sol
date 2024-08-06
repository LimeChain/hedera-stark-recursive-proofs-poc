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

import {Script} from "lib/forge-std/src/Script.sol";
import {console2} from "lib/forge-std/src/console2.sol";
import {IRiscZeroVerifier} from "lib/risc0-ethereum/contracts/src/IRiscZeroVerifier.sol";
import {RiscZeroGroth16Verifier} from "lib/risc0-ethereum/contracts/src/groth16/RiscZeroGroth16Verifier.sol";
import {ControlID} from "lib/risc0-ethereum/contracts/src/groth16/ControlID.sol";

import {CommitmentVerification} from "../contracts/CommitmentVerification.sol";

/// @notice Deployment script for the RISC Zero starter project.
/// @dev Use the following environment variable to control the deployment:
///     * ETH_WALLET_PRIVATE_KEY private key of the wallet to be used for deployment.
///
/// See the Foundry documentation for more information about Solidity scripts.
/// https://book.getfoundry.sh/tutorials/solidity-scripting
contract ProofsDeploy is Script {
    function run() external {
        uint256 deployerKey = uint256(vm.envBytes32("ETH_WALLET_PRIVATE_KEY"));

        vm.startBroadcast(deployerKey);

        IRiscZeroVerifier verifier = new RiscZeroGroth16Verifier(ControlID.CONTROL_ROOT, ControlID.BN254_CONTROL_ID);
        console2.log("Deployed RiscZeroGroth16Verifier to", address(verifier));

        CommitmentVerification commitmentVerification = new CommitmentVerification(verifier);
        console2.log("Deployed CommitmentVerification to", address(commitmentVerification));

        vm.stopBroadcast();
    }
}
