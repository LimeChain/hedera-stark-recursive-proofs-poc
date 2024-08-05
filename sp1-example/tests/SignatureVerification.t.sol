// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Test, console} from "../../lib/forge-std/src/Test.sol";
import {stdJson} from "../../lib/forge-std/src/StdJson.sol";
import {SignatureVerification} from "../contracts/SignatureVerification.sol";
import {SP1VerifierGateway} from "../../lib/sp1-contracts/contracts/src/SP1VerifierGateway.sol";

struct SP1ProofFixtureJson {
    bytes pubkey0;
    bytes pubkeyI;
    bytes proof;
    bytes publicValues;
    bytes32 vkey;
}

contract SignatureVerificationTest is Test {
    using stdJson for string;

    address verifier;
    SignatureVerification public signatureVerification;

    function loadFixture() public view returns (SP1ProofFixtureJson memory) {
        string memory root = vm.projectRoot();
        string memory path = string.concat(root, "/examples/fixture.json");
        string memory json = vm.readFile(path);
        bytes memory jsonBytes = json.parseRaw(".");
        return abi.decode(jsonBytes, (SP1ProofFixtureJson));
    }

    function setUp() public {
        SP1ProofFixtureJson memory fixture = loadFixture();

        verifier = address(new SP1VerifierGateway(address(1)));
        signatureVerification = new SignatureVerification(verifier, fixture.vkey);
    }

    function test_validateSignatureProof() public {
        SP1ProofFixtureJson memory fixture = loadFixture();

        vm.mockCall(verifier, abi.encodeWithSelector(SP1VerifierGateway.verifyProof.selector), abi.encode(true));

        (bytes memory pubkey0, bytes memory pubkeyI) = signatureVerification.verifySignatureProof(fixture.publicValues, fixture.proof);

        assertEq(pubkey0, fixture.pubkey0);
        assertEq(pubkeyI, fixture.pubkeyI);
    }
}
