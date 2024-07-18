// use risc0_zkvm::{default_prover, ExecutorEnv, ProverOpts, Receipt, VerifierContext};

// use ark_bn254::{Fr, G1Projective, G2Projective};
// use ark_ec::Group;
// use ark_ff::PrimeField;
// use ark_serialize::CanonicalSerialize;
// use ark_std::UniformRand;
// use methods::{PROOFS_ELF, PROOFS_ID};
// use sha2::{Digest, Sha384};

use ethers::abi::Token;
use risc0_zkvm::{
    sha::Digestible, stark_to_snark, Groth16Receipt, Groth16ReceiptVerifierParameters, Receipt,
    ReceiptClaim,
};
// use std::fs::File;
use anyhow::Context;
use std::io::Write;

pub type SignatureInput = (Vec<u8>, Vec<u8>, Vec<u8>);

pub fn main() -> Result<(), anyhow::Error> {
    let receipt_bytes = std::fs::read_to_string("methods/examples/receipt.json")?;
    let receipt: Receipt = serde_json::from_str(&receipt_bytes)?;

    // Encode the seal with the selector.
    let succinct_receipt = receipt.inner.succinct()?;
    let seal_bytes = succinct_receipt.get_seal_bytes();

    let seal = stark_to_snark(&seal_bytes)?.to_vec();
    // let groth16_receipt: Groth16Receipt<ReceiptClaim> = Groth16Receipt::new(
    //     seal.clone(),
    //     succinct_receipt.claim.clone(),
    //     Groth16ReceiptVerifierParameters::default().digest(),
    // );

    // Extract the journal from the receipt.
    let journal = receipt.journal.bytes.clone();

    let calldata = vec![Token::Bytes(journal), Token::Bytes(seal)];
    let output = hex::encode(ethers::abi::encode(&calldata));

    // Forge test FFI calls expect hex encoded bytes sent to stdout
    print!("{output}");
    std::io::stdout()
        .flush()
        .context("failed to flush stdout buffer")?;

    Ok(())
}
