use anyhow::Context;
use ark_bn254::{Fr, G1Projective, G2Projective};
use ark_ec::Group;
use ark_ff::PrimeField;
use ark_serialize::{CanonicalSerialize, Write};
use ark_std::UniformRand;
use ethers::abi::Token;
use methods::{PROOFS_ELF, PROOFS_ID};
use risc0_ethereum_contracts::groth16;
use risc0_zkvm::{default_prover, ExecutorEnv, Prover, ProverOpts, Receipt};
use sha2::{Digest, Sha384};
use std::{env, fs::File};

pub type SignatureInput = (Vec<u8>, Vec<u8>, Vec<u8>);

pub fn main() -> Result<(), anyhow::Error> {
    if env::var("RUN_DEMO").is_ok() {
        run_demo()?;
    } else {
        run_prover()?;
    }

    Ok(())
}

fn generate_inputs(pk: Option<Vec<u8>>) -> Result<crate::SignatureInput, anyhow::Error> {
    let g1_gen: G1Projective = G1Projective::generator();
    let g2_gen: G2Projective = G2Projective::generator();

    let mut rng = ark_std::test_rng();

    let privkey_new = Fr::rand(&mut rng);
    let pubkey_new: G2Projective = g2_gen * privkey_new;

    let mut pubkey_new_bytes: Vec<u8> = Vec::new();
    pubkey_new
        .serialize_compressed(&mut pubkey_new_bytes)
        .unwrap();

    let mut hasher = Sha384::new();
    let message_bytes = if let Some(pk_value) = pk {
        pk_value
    } else {
        let privkey_old = Fr::rand(&mut rng);
        let pubkey_old: G2Projective = g2_gen * privkey_old;
        let mut pubkey_old_bytes: Vec<u8> = Vec::new();
        pubkey_old
            .serialize_compressed(&mut pubkey_old_bytes)
            .unwrap();

        pubkey_old_bytes
    };
    hasher.update(message_bytes);
    let message_hash = hasher.finalize();
    let g1_message: G1Projective = g1_gen * Fr::from_le_bytes_mod_order(message_hash.as_slice());
    let mut g1_message_bytes: Vec<u8> = Vec::new();
    g1_message
        .serialize_compressed(&mut g1_message_bytes)
        .unwrap();

    let signature: G1Projective = g1_message * privkey_new;

    let mut signature_bytes: Vec<u8> = Vec::new();
    signature
        .serialize_compressed(&mut signature_bytes)
        .unwrap();

    Ok((pubkey_new_bytes, g1_message_bytes, signature_bytes))
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
    // Fetch assumption receipt path passed as inline argument
    let arg = env::args().next();

    // Initialize the prover
    let prover: std::rc::Rc<dyn Prover> = default_prover();

    // Check if the assumption receipt path is passed as an argument
    // If not, generate a new assumption receipt
    // If yes, read the receipt from the path and fetch the public key from the journal
    // pubkey_0 is the public key of the first epoch
    let (pubkey_0, assumption_receipt) = match arg {
        None => generate_assumption(&prover)?,
        Some(receipt_path) => {
            let assumption_receipt: Receipt = read_receipt(&receipt_path)?;
            let pubkey_0: Vec<u8> = assumption_receipt.journal.decode()?;
            (pubkey_0, assumption_receipt)
        }
    };

    // Generate the composite inputs
    let composite_inputs: SignatureInput = generate_inputs(Some(pubkey_0.clone()))?;
    // Create the environment for the composition circuit
    let env_1 = ExecutorEnv::builder()
        .add_assumption(assumption_receipt)
        .write(&composite_inputs)
        .unwrap()
        .write(&Some(pubkey_0))
        .unwrap()
        .write(&PROOFS_ID)
        .unwrap()
        .build()
        .unwrap();

    // Prove the composition circuit
    let prove_info = prover.prove(env_1, PROOFS_ELF)?;
    let composition_receipt = prove_info.receipt;
    // Save the composition receipt to a json file
    save_receipt(&composition_receipt, "composition_receipt")?;

    composition_receipt.verify(PROOFS_ID)?;

    // TODO: Check if we can compress the receipts in a single step from composit to groth16
    // Compress the receipts to succinct
    let succinct_receipt = prover.compress(&ProverOpts::succinct(), &composition_receipt)?;

    save_receipt(&succinct_receipt, "succinct_receipt")?;

    succinct_receipt.verify(PROOFS_ID)?;
    // Compress the receipts to groth16
    let groth16_receipt = prover.compress(&ProverOpts::groth16(), &succinct_receipt)?;

    let _ = save_receipt(&groth16_receipt, "groth16_receipt");

    groth16_receipt.verify(PROOFS_ID)?;

    // Send the calldata to the forge test FFI
    send_calldata(groth16_receipt)?;

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

    send_calldata(groth16_receipt)?;

    Ok(())
}

fn read_receipt(file_name: &str) -> Result<Receipt, anyhow::Error> {
    let receipt: Receipt = serde_json::from_str(&std::fs::read_to_string(format!(
        "receipts/{file_name}.json"
    ))?)?;
    Ok(receipt)
}

fn send_calldata(groth16_receipt: Receipt) -> Result<(), anyhow::Error> {
    let journal = groth16_receipt.journal.bytes.clone();
    let groth16 = groth16_receipt.inner.groth16()?;
    let seal = groth16.seal.clone();
    let encoded_seal = groth16::encode(seal)?;

    let calldata = vec![Token::Bytes(journal), Token::Bytes(encoded_seal)];
    let output = hex::encode(ethers::abi::encode(&calldata));

    // Forge test FFI calls expect hex encoded bytes sent to stdout
    print!("{output}");
    std::io::stdout()
        .flush()
        .context("failed to flush stdout buffer")?;

    Ok(())
}

fn generate_assumption(
    prover: &std::rc::Rc<dyn Prover>,
) -> Result<(Vec<u8>, Receipt), anyhow::Error> {
    let (pubkey_0, msg_0, sig_0) = generate_inputs(None)?;

    let no_pk: Option<Vec<u8>> = None;
    let env_0 = ExecutorEnv::builder()
        .write(&(pubkey_0.clone(), msg_0, sig_0))
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

    Ok((pubkey_0, assumption_receipt))
}
