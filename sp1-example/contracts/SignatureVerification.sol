// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {ISP1Verifier} from "lib/sp1-contracts/contracts/src/ISP1Verifier.sol";

/// @title signature.
/// @author Succinct Labs
/// @notice This contract implements a simple example of verifying the proof of a computing a
///         signature number.
contract SignatureVerification {
    /// @notice The address of the SP1 verifier contract.
    /// @dev This can either be a specific SP1Verifier for a specific version, or the
    ///      SP1VerifierGateway which can be used to verify proofs for any version of SP1.
    ///      For the list of supported verifiers on each chain, see:
    ///      https://github.com/succinctlabs/sp1-contracts/tree/main/contracts/deployments
    address public verifier;

    /// @notice The verification key for the proofs program.
    bytes32 public proofsProgramVK;

    constructor(address _verifier, bytes32 _proofsProgramVK) {
        verifier = _verifier;
        proofsProgramVK = _proofsProgramVK;
    }

    /// @notice The entrypoint for verifying the proof of a signature verification.
    /// @param _proofBytes The encoded proof.
    /// @param _publicValues The encoded public values.
    function verifySignatureProof(bytes calldata _publicValues, bytes calldata _proofBytes)
        public
        view
        returns (bool)
    {
        ISP1Verifier(verifier).verifyProof(proofsProgramVK, _publicValues, _proofBytes);

        return true;
    }
}
