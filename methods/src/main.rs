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
use risc0_zkvm::{get_prover_server, recursion::identity_p254, stark_to_snark, ExecutorEnv, ProverOpts, Receipt, ReceiptClaim, SuccinctReceipt};

use sha2::{Digest, Sha384};

pub type SignatureInput = (Vec<u8>, Vec<u8>, Vec<u8>);

pub fn main() -> Result<(), anyhow::Error> {
    println!("Generate inputs for epoch 0 receipt");
    let (pk_0, msg_0, sig_0) = generate_inputs(None)?;

    let opts = ProverOpts::default();
    let prover = get_prover_server(&opts)?;
    let no_pk: Option<Vec<u8>> = None;
    println!("Build environment for epoch 0 receipt");
    let env_0 = ExecutorEnv::builder()
        .write(&(pk_0.clone(), msg_0, sig_0))
        .unwrap()
        .write(&no_pk)
        .unwrap()
        .write(&PROOFS_ID)
        .unwrap()
        .build()
        .unwrap();

    println!("Create assumption receipt for epoch 0");
    let assumption_receipt = prover.prove(env_0, PROOFS_ELF)?.receipt;

    println!("Generate inputs for epoch 1 receipt");
    let composite_inputs: SignatureInput = generate_inputs(Some(pk_0.clone()))?;
    println!("Build environment for epoch 1 receipt");
    let env_1 = ExecutorEnv::builder()
        .add_assumption(assumption_receipt)
        .write(&composite_inputs)
        .unwrap()
        .write(&Some(pk_0))
        .unwrap()
        .write(&PROOFS_ID)
        .unwrap()
        .build()
        .unwrap();

    println!("Prove the receipt for epoch 0 and signature for epoch 1");
    let prove_info = prover.prove(env_1, PROOFS_ELF)?;
    let composition_receipt = prove_info.receipt;

    // Encode the seal with the selector.
    println!("Compress the receipt");
    let succinct_receipt = prover.compress(&ProverOpts::default(), &composition_receipt)?;
    println!("Extract Journal");
    let journal = succinct_receipt.journal.clone();
    println!("Convert to identity_p254");
    let ident_receipt: SuccinctReceipt<ReceiptClaim> = identity_p254(succinct_receipt.inner.succinct()?).unwrap();
    let seal_bytes = ident_receipt.get_seal_bytes();
    println!("Convert to snark");
    let seal = stark_to_snark(&seal_bytes)?.to_vec();

    println!("Prepare the output for on-chain verification");
    let calldata = vec![Token::Bytes(journal.bytes), Token::Bytes(seal)];
    let output = hex::encode(ethers::abi::encode(&calldata));

    // Forge test FFI calls expect hex encoded bytes sent to stdout
    print!("Send output: {output}");
    std::io::stdout()
        .flush()
        .context("failed to flush stdout buffer")?;

    Ok(())
}

fn generate_inputs(pk: Option<Vec<u8>>) -> Result<crate::SignatureInput, anyhow::Error> {
    let g1_gen: G1Projective = G1Projective::generator();
    let g2_gen: G2Projective = G2Projective::generator();

    let mut rng = ark_std::test_rng();
    let s1 = Fr::rand(&mut rng);
    let s2 = Fr::rand(&mut rng);

    let pk_old: G2Projective = g2_gen * s1;
    let pk_new: G2Projective = g2_gen * s2;

    let mut pk_old_bytes: Vec<u8> = Vec::new();
    pk_old.serialize_compressed(&mut pk_old_bytes).unwrap();

    let mut pk_new_bytes: Vec<u8> = Vec::new();
    pk_new.serialize_compressed(&mut pk_new_bytes).unwrap();

    let mut hasher = Sha384::new();
    let msg_bytes = if pk.is_none() { pk_old_bytes } else  { pk.unwrap() };
    hasher.update(msg_bytes);
    let message_hash = hasher.finalize();
    let message: G1Projective = g1_gen * Fr::from_le_bytes_mod_order(message_hash.as_slice());
    let mut message_bytes: Vec<u8> = Vec::new();
    message.serialize_compressed(&mut message_bytes).unwrap();

    let signature: G1Projective = message * s2;

    let mut signature_bytes: Vec<u8> = Vec::new();
    signature
        .serialize_compressed(&mut signature_bytes)
        .unwrap();

    Ok((pk_new_bytes, message_bytes, signature_bytes))
}
