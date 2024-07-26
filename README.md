# Hedera STARK recursive proof using RISC Zero

## Dependencies

Install `Rust` and `Foundry`, and then restart your terminal.

```sh
# Install Rust
curl https://sh.rustup.rs -sSf | sh
# Install Foundry
curl -L https://foundry.paradigm.xyz | bash
```

Next, you will need to install the `cargo risczero` tool.
We'll use [`cargo binstall`][cargo-binstall] to get `cargo-risczero` installed, and then install the `risc0` toolchain.
See [RISC Zero installation] for more details.

```sh
cargo install cargo-binstall
cargo binstall cargo-risczero
cargo risczero install
```

You'll also need to have docker installed on your machine. You can find the installation instructions [here](https://docs.docker.com/get-docker/).

### CUDA support
For running on CUDA-Enabled GPUs, you will need to install the [cuda toolkit](https://docs.nvidia.com/cuda/#installation-guides).

### Build the Code

- Build the Rust code

  ```sh
  cargo build --release
  ```

- Build the Rust code for CUDA-Enabled GPUs

  ```sh
  cargo build --release --F cuda
  ```

- Build your Solidity smart contracts

  > NOTE: `cargo build` needs to run first to generate the `ImageID.sol` contract.

  ```sh
  forge build
  ```


### Deterministic Builds

By setting the environment variable `RISC0_USE_DOCKER` a containerized build process via Docker will ensure that all builds of your guest code, regardless of the machine or local environment, will produce the same [image ID][image-id].

```sh
RISC0_USE_DOCKER=1 cargo build --release
```

### Run the Code

NOTE: The receipts in the `examples.zip` archive have been generated on a VM using CUDA-Enabled GPUS. In order to be able to validate them on a local machine the generated `ImageID` should be the same as the one used to generate the receipts. The receipts were generating using docker as described in the previous section. For this reason use the `RISC0_USE_DOCKER=1` flag when running the tests using the example receipts.

- Run the Rust code on a local machine. Note: This will take several hours to produce the Groth16 SNARK proofs for the zkVM prover.

  ```sh
    RISC0_DEV_MODE=false cargo run --release
  ```

- Run the Rust code using CUDA-Enabled GPUs Acceleration. Note: Requires a compatible GPU and the CUDA Toolkit installed.

  ```sh
    RISC0_DEV_MODE=false cargo run --release --F cuda
  ```

- Run the rust code with previously generated assumption receipts. Note: Extract the `examples.zip` archive containing the receipts in the root folder of the project. You can find the receipts in the `examples` folder. Pass the file name as an argument to the command. Groth16 receipts cannot be used as assumptions.
  ```sh
    RISC0_USE_DOCKER=1 RISC0_DEV_MODE=false cargo run --release -- assumption_receipt
  ```

- Run the Rust code in demo mode. Note: This will read previously generated receipts for the on-chain verification. Note: Extract the `examples.zip` archive containing the receipts in the root folder of the project. For Deterministic Builds, set the `RISC0_USE_DOCKER` environment variable.

  ```sh
    RISC0_USE_DOCKER=1 RISC0_DEV_MODE=false RUN_DEMO=true cargo run --release
  ```

### Run the Tests

- Run the tests, with the full zkVM prover. Note: Producing the [Groth16 SNARK proofs][Groth16] for this test requires running on an x86 machine with CUDA-Enabled GPUs and [Docker] installed. Apple silicon is currently unsupported for local proving, you can find out more info in the relevant issues [here](https://github.com/risc0/risc0/issues/1520) and [here](https://github.com/risc0/risc0/issues/1749).

  ```sh
    RISC0_USE_DOCKER=1 RISC0_DEV_MODE=false forge test --match-path tests/CommitmentVerificationDemo.t.sol -vvv
  ```

- For Apple silicon, there's a demo mode that can be used to run the tests without producing proofs. It will read previously generated receipts for the on-chain verification.

  Extract the `examples.zip` archive containing the receipts in the root folder of the project and run the following command:

  ```sh
    RISC0_USE_DOCKER=1 RUN_DEMO=true RISC0_DEV_MODE=false forge test --match-path tests/CommitmentVerificationDemo.t.sol -vvv
  ```
