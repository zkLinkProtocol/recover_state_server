# ZkLink Exodus Model

The Rust Implementation of the ZkLink exodus server and prover.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Getting Started](#getting-started)
    - [Create the Database](#create-the-database)
    - [Build the Project](#build-the-project)
    - [Configure the Environment Variables](#configure-the-environment-variables)
    - [Recover the Database](#recover-ZkLink-state)
    - [Start the Server](#start-the-exodus-server)
    - [Start the Prover](#start-the-exodus-prover)
- [Contributing](#contributing)
- [License](#license)

## Prerequisites

Before you begin, you will need to have the following software installed:

- [Rust and rustup](https://www.rust-lang.org/tools/install).
- [PostgreSQL](https://www.postgresql.org/download/).
- [Diesel](http://diesel.rs/) command-line tool for Rust. You can install it by running:
```
cargo install diesel_cli --no-default-features --features postgres
```

## Getting Started

### Create the Database

To create the database, run the following command in the `storage` directory:
```
diesel setup
```
This will create the necessary tables in your PostgreSQL database.

### Build the Project

To build the project in release mode, run the following command:
```
cargo build --release
```
This will create a binary file in the `target/release` directory.

### Configure the Environment Variables
Your need to modify the following configuration:

1. `CLIENT_WEB3_URL` for `CHAIN_1` and `CHAIN_2`: the url of the rpc node of the corresponding chain.
2. `CONTRACT_BLOCK_HEIGHT` for `CHAIN_1` and `CHAIN_2`: the block height of the zklink contract depolyment.
3. `CONTRACT_ADDR` for `CHAIN_1` and `CHAIN_2`: the contract address of zklink contract.
4. `CONTRACT_GENESIS_TX_HASH` for `CHAIN_1` and `CHAIN_2`: the transaction hash of the zklink contract depolyment.
5. `FULLY_ON_CHAIN` for `CHAIN_1` and `CHAIN_2`: it means whether the data is fully on-chain.
6. `DATABASE_URL`: the default is local.

### Recover ZkLink state
To recover the database, run the following command:
```
./target/release/exduos_server --recover
```
This command will take several hours to complete. Please be patient and wait until the command finishes.

### Start the exodus Server
To start the server, run the following command:
```
./target/release/exduos_server --server
```

### Start the exodus Prover
To start the prover and generate a proof for the server to receive a create proof command, run the following command:
```
./target/release/exduos_prover
```

## Contributing
Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for more information.

## License
This project is licensed under the [MIT License](LICENSE).