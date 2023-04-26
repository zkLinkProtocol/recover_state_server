# zkLink Exodus Model
The Server, Prover and React App of ZkLink Exodus Model.

## Table of Contents
- [Prerequisites](#prerequisites)
- [Getting Started](#getting-started)
    - Backend
        - [Download the setup](#Download-the-setup)
        - [Create the Database](#create-the-database)
        - [Build the Project](#build-the-project)
        - [Configure the Environment Variables](#configure-the-environment-variables)
        - [Recover zklink State](#Recover-ZkLink-State)
        - [Start Exodus Server and Exodus Prove](#Start-Exodus-Server-and-Exodus-Prove)
    - [Frontend setup](exodus-interface/README.md)
- [License](#license)

## Prerequisites

You need to select a suitable server first, and we provide three recommended configurations.

| AWS EC2 Instance | Price     | Prove Performance |
|------------------|-----------|-------------------|
| c5a.4xlarge      | $0.768/hr | 1.6 proofs/min    |
| c5a.12xlarge     | $2.304/hr | 4 proofs/min      |
| c5a.24xlarge     | $4.608/hr | 5.5 proofs/min    |

First, install basic lib:
```shell
sudo apt-get install libpq-dev libssl-dev pkg-config axel
curl -qL https://www.npmjs.com/install.sh | sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
Before you begin, you will need to have the following software installed:
- [PostgreSQL](https://www.postgresql.org/download/linux/ubuntu/) database. 
- [Diesel](http://diesel.rs/) command-line tool for Rust. You can install it by running:
```shell
cargo install diesel_cli --no-default-features --features postgres
```
Load git repository:
```shell
git clone https://github.com/zkLinkProtocol/recover_state_server.git
```

## Getting Started
### Download the setup
Run the following command in the `zklink_keys` directory:
```shell
axel -c https://universal-setup.ams3.digitaloceanspaces.com/setup_2%5E21.key
```
### Create the Database
First, to configure the `DATABASE_URL` environment:
```shell
export DATABASE_URL=postgres://postgres:password@localhost/plasma
```
For the first time, please refer to [psql.md](docs/psql.md) for setting the password.

### Configure the Environment Variables
First, there is a `.env.eg` file in the root path of our project, copy and rename it to `.env`.
```shell
cp .env.e.g .env
```
Explanation of .env Configuration Items: [env.md](env.md)

Then, you need to modify the following configuration:

**note:**
1. `RUNTIME_CONFIG_ZKLINK_HOME` and `DATABASE_URL` must be configured by your current environment

```shell
export DATABASE_URL=postgres://user:password@localhost/plasma
export RUNTIME_CONFIG_ZKLINK_HOME = /home/xxx_user/recover_state_server 

```
2. Before "dunkerque," a link will be published here that will display contract and chain configurations. if test, only use default.


-----
### Recover ZkLink state
To recover the state, run the following `start` command:
```shell
./exodus.sh start
```
This command will take several hours to complete.

If you want to see the recover state process, please:
```shell
tail -f log/recover_state.log
```
**The program will close automatically when synchronization is complete.**

### Close Exodus Server and Exodus Prove
```shell
./exodus.sh stop
```

### Clean Up Exodus All Data
```shell
./exodus.sh clean
```

### Frontend setup
Please refer to [Frontend setup](exodus-interface/README.md)

## License
This project is licensed under the MIT License - see the `LICENSE` file for details.
