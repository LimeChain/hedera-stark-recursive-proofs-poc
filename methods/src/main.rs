// use risc0_zkvm::{default_prover, ExecutorEnv, ProverOpts, Receipt, VerifierContext};

// use ark_bn254::{Fr, G1Projective, G2Projective};
// use ark_ec::Group;
// use ark_ff::PrimeField;
// use ark_serialize::CanonicalSerialize;
// use ark_std::UniformRand;
// use methods::{PROOFS_ELF, PROOFS_ID};
// use sha2::{Digest, Sha384};

use anyhow::Context;
use ethers::abi::Token;
use methods::{PROOFS_ELF, PROOFS_ID};
use ark_bn254::{Fr, G1Projective, G2Projective};
use ark_ec::Group;
use ark_ff::PrimeField;
use ark_serialize::{CanonicalSerialize, Write};
use ark_std::UniformRand;
use risc0_zkvm::{default_prover, stark_to_snark, ExecutorEnv, Receipt};
use sha2::{Digest, Sha384};

pub type SignatureInput = (Vec<u8>, Vec<u8>, Vec<u8>);

pub fn main() -> Result<(), anyhow::Error> {
    let receipt_bytes = std::fs::read_to_string("methods/examples/receipt.json")?;
    let receipt: Receipt = serde_json::from_str(&receipt_bytes)?;
    let previous_pk: Vec<u8> = receipt.journal.decode().unwrap();

    let g1_gen: G1Projective = G1Projective::generator();
    let g2_gen: G2Projective = G2Projective::generator();

    let mut rng = ark_std::test_rng();
    let s1 = Fr::rand(&mut rng);
    let pk_new: G2Projective = g2_gen * s1;
    let mut pk_new_bytes: Vec<u8> = Vec::new();
    pk_new.serialize_compressed(&mut pk_new_bytes).unwrap();

    let mut hasher = Sha384::new();
    hasher.update(&previous_pk);
    let message_hash = hasher.finalize();
    let message: G1Projective = g1_gen * Fr::from_le_bytes_mod_order(message_hash.as_slice());
    let mut message_bytes: Vec<u8> = Vec::new();
    message.serialize_compressed(&mut message_bytes).unwrap();

    let signature: G1Projective = message * s1;

    let mut signature_bytes: Vec<u8> = Vec::new();
    signature
        .serialize_compressed(&mut signature_bytes)
        .unwrap();

    let signature_input: SignatureInput = (pk_new_bytes, message_bytes, signature_bytes);

    let env = ExecutorEnv::builder()
        .add_assumption(receipt)
        .write(&signature_input)
        .unwrap()
        .write(&Some(previous_pk))
        .unwrap()
        .write(&PROOFS_ID)
        .unwrap()
        .build()
        .unwrap();


    let prover = default_prover();
    let proof = prover
        .prove(env, PROOFS_ELF)
        .unwrap();
    let receipt = proof.receipt;

    receipt.verify(PROOFS_ID).unwrap();
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
