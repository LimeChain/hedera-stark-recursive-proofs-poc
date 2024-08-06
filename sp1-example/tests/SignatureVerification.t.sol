// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Test, console} from "lib/forge-std/src/Test.sol";
import {stdJson} from "lib/forge-std/src/StdJson.sol";
import {SignatureVerification} from "../contracts/SignatureVerification.sol";
import {SP1Verifier} from "lib/sp1-contracts/contracts/src/v1.0.1/SP1Verifier.sol";

struct SP1ProofFixtureJson {
    bytes proof;
    bytes publicValues;
    bytes32 vkey;
}

contract SignatureVerificationTest is Test {
    using stdJson for string;

    address verifier;
    SignatureVerification public signatureVerification;
    SP1ProofFixtureJson fixture;

    function loadFixture() public view returns (SP1ProofFixtureJson memory) {
        string memory root = vm.projectRoot();
        string memory path = string.concat(root, "/examples/fixture.json");
        string memory json = vm.readFile(path);
        bytes memory jsonBytes = json.parseRaw(".");
        return abi.decode(jsonBytes, (SP1ProofFixtureJson));
    }

    function setUp() public {
        fixture = loadFixture();

        verifier = address(new SP1Verifier());
        signatureVerification = new SignatureVerification(verifier, fixture.vkey);
    }

    function test_validateSignatureProof() public view {
        signatureVerification.verifySignatureProof(fixture.publicValues, fixture.proof);
    }
}
