# zkLink Exodus Model
The Server, Prover and React App of ZkLink Exodus Model.

## Table of Contents
- [Prerequisites](#prerequisites)
- [Getting Started](#getting-started)
    - [Create the Database](#create-the-database)
    - [Build the Project](#build-the-project)
    - [Configure the Environment Variables](#configure-the-environment-variables)
    - [Recover zklink state](#recover-ZkLink-state)
    - [Start the Server](#start-the-exodus-server)
    - [Start the Prover](#start-the-exodus-prover)
    - [Front-end setup](exodus-interface/README.md)
- [Contributing](#contributing)
- [License](#license)

## Prerequisites

You need to select a suitable server first, and we provide three recommended configurations.
| AWS EC2 Instance | Price | Prove Performance |
| ---------------- | ----- | ----------------- |
| c5a.4xlarge         | $0.768/hr | 1.6 proofs/min |
| c5a.12xlarge         | $2.304/hr | 4 proofs/min |
| c5a.24xlarge        | $4.608/hr | 5.5 proofs/min |


Before you begin, you will need to have the following software installed:

- [Rust and rustup](https://www.rust-lang.org/tools/install).
- [PostgreSQL](https://www.postgresql.org/download/).
- [Diesel](http://diesel.rs/) command-line tool for Rust. You can install it by running:
```shell
cargo install diesel_cli --no-default-features --features postgres
```
Load git repository:
```shell
git clone --recursive https://github.com/zkLinkProtocol/recover_state_server.git
```

## Getting Started
### Download the setup
Run the following command in the `zklink_keys` directory:
```shell
axel -c https://universal-setup.ams3.digitaloceanspaces.com/setup_2%5E21.key
```
### Create the Database
First, You need to configure the `DATABASE_URL=postgres://user:password@localhost/plasma` environment.

Then, to create the database, run the following command in the `storage` directory:
```shell
diesel database setup
```
This will create the necessary tables in your PostgreSQL database.

### Build the Project

To build the project in release mode, run the following command:
```shell
cargo build --release
```
This will create a binary file in the `target/release` directory.

### Configure the Environment Variables
First, there is a `.env.eg` file in the root path of our project, copy and rename it to `.env`.
```shell
cp .env.e.g .env
```
Then, you need to modify the following configuration:
(Before "dunkerque," a link will be published here that will display all configurations except for DATABASE_URL.)

1. `CLIENT_WEB3_URL` for `CHAIN_1` and `CHAIN_2`: the url of the rpc node of the corresponding chain.
2. `CONTRACT_BLOCK_HEIGHT` for `CHAIN_1` and `CHAIN_2`: the block height of the zklink contract depolyment.
3. `CONTRACT_ADDR` for `CHAIN_1` and `CHAIN_2`: the contract address of zklink contract.
4. `CONTRACT_GENESIS_TX_HASH` for `CHAIN_1` and `CHAIN_2`: the transaction hash of the zklink contract depolyment.
5. `FULLY_ON_CHAIN` for `CHAIN_1` and `CHAIN_2`: it means whether the data is fully on-chain.
6. `DATABASE_URL`: the default is local.

### Recover ZkLink state
To recover the state, run the following `genesis` command:
```shell
./target/debug/recover_state --genesis
```
This command will take several hours to complete. **Please be patient and wait until the command finishes.**

If there is an interruption, run the `continue` command
```shell
./target/debug/recover_state --continue
```


### Start Exodus Server and Exodus Prove
To start the server, run the following command:
```
./target/release/exodus_server
```
To start the prover and generate a proof for the server to receive a create proof command, run the following command:
```
./target/release/exduos_prover tasks
```
Please refer to prover [README.md](prover/README.md) for detailed command details


## License
This project is licensed under the MIT License - see the `LICENSE` file for details.
