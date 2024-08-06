# SP1 Hedera recursive proof using example

This is an example end-to-end [SP1](https://github.com/succinctlabs/sp1) project
that can generate a proof of any RISC-V program.

## Requirements

- [Rust](https://rustup.rs/)
- [SP1](https://succinctlabs.github.io/sp1/getting-started/install.html)

### Build the Code

- Build the Rust code

  ```sh
  cargo build --release
  ```

- Build your Solidity smart contracts

  ```sh
  forge build
  ```


## Running the Rust code

### Standard Proof Generation

> [!WARNING]
> You will need at least 16GB RAM to generate the default proof.

Generate the proof for your program using the standard prover.

```sh
cd script
RUST_LOG=info cargo run --release -- --pubkey0 0x1234....
```

### EVM-Compatible Proof Generation & Verification

> [!WARNING]
> You will need at least 128GB RAM to generate the PLONK proof.

Generate the proof that is small enough to be verified on-chain and verifiable by the EVM. This command also generates a fixture that can be used to test the verification of SP1 zkVM proofs inside Solidity.

```sh
cd script
RUST_LOG=info cargo run --release -- --pubkey0 0x1234.... --plonk
```

### Using the Prover Network

Make a copy of the example environment file:

```sh
cp .env.example .env
```

Then, set the `SP1_PROVER` environment variable to `network` and set the `SP1_PRIVATE_KEY` environment variable to your whitelisted private key. For more information, see the [setup guide](https://docs.succinct.xyz/prover-network/setup.html).

> [!NOTE]
> The code was tested on the `network` prover, so the examples bellow will be shown using the `network` prover.

- Run the rust code WITHOUT previously generated proofs. This workflow will generate a compressed proof first, that will be used as an "assumption" for the final proof.
  ```sh
  cd script
  SP1_PROVER=network SP1_PRIVATE_KEY=0x12345... RUST_LOG=info cargo run --release -- --pubkey0 0x12345...
  ```

- Run the rust code with previously generated proofs. This workflow will generate just the final compressed proof that can be used as assumption in the next epoch. You can find the proofs in the `examples` folder. Pass the file path as an argument to the command. PLONK receipts cannot be used.
  ```sh
  cd script
  SP1_PROVER=network SP1_PRIVATE_KEY=0x12345... RUST_LOG=info cargo run --release -- --pubkey0 0x12345... --receipts path/to/receipt.json
  ```

- Generate a PLONK proof that can be verified on-chain. This command also generates a fixture that can be used to test the verification of SP1 zkVM proofs inside Solidity. Because conversion from compressed to PLONK requires at least 128GB RAM, in the example we use the newtork prover to generate two proofs, one compressed and one PLONK.

  ```sh
  cd script
  SP1_PROVER=network SP1_PRIVATE_KEY=0x12345... RUST_LOG=info cargo run --release -- --pubkey0 0x12345... --plonk
  ```

- Run the Rust code in demo mode. Demo mode will use the previously generated PLONK proof from the `examples` folder to generate a fixture that can be used to test the verification of SP1 zkVM proofs inside Solidity.
  ```sh
  cd script
  cargo run --release -- --demo
  ```

## Run the Tests

- The example contains a test for on-chain verification of the SP1 zkVM proof. It uses the fixture generated in the `examples` folder by executing either the `demo` flow with a previously generated proof or after generating a PLONK proof with the main flow using the `--plonk` flag.

> [!NOTE]
> The `examples` folder already contains a `fixture.json` file that can be used in the test.

```sh
forge test --match-path tests/SignatureVerification.t.sol -vvv
```
