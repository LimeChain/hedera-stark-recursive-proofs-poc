use anyhow::Context;
use ark_bn254::{Fr, G1Projective, G2Projective};
use ark_ec::Group;
use ark_ff::PrimeField;
use ark_serialize::{CanonicalSerialize, Write};
use ark_std::UniformRand;
use ethers::abi::Token;
use methods::{PROOFS_ELF, PROOFS_ID};
use risc0_ethereum_contracts::groth16::{self, encode};
use risc0_zkvm::{
    default_prover, recursion::identity_p254, sha::Digestible, stark_to_snark, ExecutorEnv, Groth16Receipt, Groth16ReceiptVerifierParameters, InnerReceipt, Journal, ProverOpts, Receipt, ReceiptClaim, SuccinctReceipt
};
use std::{env, fs::File};
use sha2::{Digest, Sha384};

pub type SignatureInput = (Vec<u8>, Vec<u8>, Vec<u8>);

pub fn main() -> Result<(), anyhow::Error> {
    let (pk_0, msg_0, sig_0) = generate_inputs(None)?;

    let prover = default_prover();
    let no_pk: Option<Vec<u8>> = None;
    let env_0 = ExecutorEnv::builder()
        .write(&(pk_0.clone(), msg_0, sig_0))
        .unwrap()
        .write(&no_pk)
        .unwrap()
        .write(&PROOFS_ID)
        .unwrap()
        .build()
        .unwrap();

    let assumption_receipt = prover.prove(env_0, PROOFS_ELF)?.receipt;
    assumption_receipt.verify(PROOFS_ID)?;

    let _ = save_receipt(&assumption_receipt, "assumption_receipt");

    let composite_inputs: SignatureInput = generate_inputs(Some(pk_0.clone()))?;
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

    let prove_info = prover.prove(env_1, PROOFS_ELF)?;
    let composition_receipt = prove_info.receipt;

    save_receipt(&composition_receipt, "composition_receipt")?;

    composition_receipt.verify(PROOFS_ID)?;
    // Encode the seal with the selector.
    let succinct_receipt = prover.compress(&ProverOpts::succinct(), &composition_receipt)?;

    save_receipt(&succinct_receipt, "succinct_receipt")?;

    succinct_receipt.verify(PROOFS_ID)?;
    // let journal_bytes = succinct_receipt.journal.bytes.clone();
    // let ident_receipt: SuccinctReceipt<ReceiptClaim> =
    //     identity_p254(succinct_receipt.inner.succinct()?).unwrap();
    // let seal_bytes = ident_receipt.get_seal_bytes();
    // let seal = stark_to_snark(&seal_bytes)?.to_vec();

    // let groth16_receipt = Receipt::new(
    //     InnerReceipt::Groth16(Groth16Receipt::new(
    //         seal.clone(),
    //         ident_receipt.claim.clone(),
    //         Groth16ReceiptVerifierParameters::default().digest(),
    //     )),
    //     journal_bytes.clone(),
    // );

    let groth16_receipt = prover::compress(ProverOpts::groth16(), &succinct_receipt)?;
    let journal_bytes = groth16_receipt.journal.bytes.clone();
    let seal = groth16_receipt.inner.groth16()?.seal.clone();

    let _ = save_receipt(&groth16_receipt, "groth16_receipt");

    let calldata = vec![Token::Bytes(journal_bytes), Token::Bytes(seal)];
    let output = hex::encode(ethers::abi::encode(&calldata));

    // Forge test FFI calls expect hex encoded bytes sent to stdout
    print!("{output}");
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
    let msg_bytes = if let Some(pk_value) = pk {
        pk_value
    } else {
        pk_old_bytes
    };
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

fn save_receipt(receipt: &Receipt, file_name: &str) -> Result<(), anyhow::Error> {
    let receipt_bytes = serde_json::to_string_pretty(&receipt).unwrap();
    std::fs::create_dir_all("./examples")?;
    let mut file = File::create(format!("examples/{file_name}.json"))?;

    // Write the serialized string to the file
    file.write_all(receipt_bytes.as_bytes())?;

    Ok(())
}

fn run_prover() -> Result<(), anyhow::Error> {
    let (pk_0, msg_0, sig_0) = generate_inputs(None)?;

    let prover = default_prover();
    let no_pk: Option<Vec<u8>> = None;
    let env_0 = ExecutorEnv::builder()
        .write(&(pk_0.clone(), msg_0, sig_0))
        .unwrap()
        .write(&no_pk)
        .unwrap()
        .write(&PROOFS_ID)
        .unwrap()
        .build()
        .unwrap();

    let assumption_receipt = prover.prove(env_0, PROOFS_ELF)?.receipt;
    assumption_receipt.verify(PROOFS_ID)?;

    let _ = save_receipt(&assumption_receipt, "assumption_receipt");

    let composite_inputs: SignatureInput = generate_inputs(Some(pk_0.clone()))?;
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

    let prove_info = prover.prove(env_1, PROOFS_ELF)?;
    let composition_receipt = prove_info.receipt;

    save_receipt(&composition_receipt, "composition_receipt")?;

    composition_receipt.verify(PROOFS_ID)?;
    // Encode the seal with the selector.
    let succinct_receipt = prover.compress(&ProverOpts::succinct(), &composition_receipt)?;

    save_receipt(&succinct_receipt, "succinct_receipt")?;

    succinct_receipt.verify(PROOFS_ID)?;
    let journal_bytes = composition_receipt.journal.bytes.clone();
    let ident_receipt: SuccinctReceipt<ReceiptClaim> =
        identity_p254(succinct_receipt.inner.succinct()?).unwrap();
    let seal_bytes = ident_receipt.get_seal_bytes();
    let seal = stark_to_snark(&seal_bytes)?.to_vec();

    let groth16_receipt = Receipt::new(
        InnerReceipt::Groth16(Groth16Receipt::new(
            seal.clone(),
            ident_receipt.claim.clone(),
            Groth16ReceiptVerifierParameters::default().digest(),
        )),
        journal_bytes.clone(),
    );

    let _ = save_receipt(&groth16_receipt, "groth16_receipt");

    let calldata = vec![Token::Bytes(journal_bytes), Token::Bytes(encode(seal)?)];
    let output = hex::encode(ethers::abi::encode(&calldata));

    // Forge test FFI calls expect hex encoded bytes sent to stdout
    print!("{output}");
    std::io::stdout()
        .flush()
        .context("failed to flush stdout buffer")?;

    Ok(())
}

fn run_demo() -> Result<(), anyhow::Error> {
    // fetch receipts from the receipts directory
    let assumption_receipt: Receipt = read_receipt("assumption_receipt")?;
    let composition_receipt: Receipt = read_receipt("composition_receipt")?;
    let succinct_receipt: Receipt = read_receipt("succinct_receipt")?;
    let groth16_receipt: Receipt = read_receipt("groth16_receipt")?;

    assumption_receipt.verify(PROOFS_ID)?;
    composition_receipt.verify(PROOFS_ID)?;
    succinct_receipt.verify(PROOFS_ID)?;
    groth16_receipt.verify(PROOFS_ID)?;

    let journal = groth16_receipt.journal.bytes.clone();
    let groth16 = groth16_receipt.inner.groth16()?;
    let seal = groth16.seal.clone();
    let encoded_seal = encode(seal)?;

    let calldata = vec![Token::Bytes(journal), Token::Bytes(encoded_seal)];
    let output = hex::encode(ethers::abi::encode(&calldata));

    // Forge test FFI calls expect hex encoded bytes sent to stdout
    print!("{output}");
    std::io::stdout()
        .flush()
        .context("failed to flush stdout buffer")?;

    Ok(())
}

fn read_receipt(file_name: &str) -> Result<Receipt, anyhow::Error> {
    let receipt: Receipt = serde_json::from_str(
        &std::fs::read_to_string(format!("receipts/{file_name}.json"))?,
    )?;
    Ok(receipt)
}
