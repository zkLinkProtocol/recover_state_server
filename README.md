# zkLink Exodus Model
This repository contains the Server, Prover and React App for zkLink Exodus Model.

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
Before getting started, you need to select a suitable server. We recommend three configurations based on the required prove performance and cost:


| AWS EC2 Instance | Price     | Prove Performance |
|------------------|-----------|-------------------|
| c5a.4xlarge      | $0.768/hr | 1.6 proofs/min    |
| c5a.12xlarge     | $2.304/hr | 4 proofs/min      |
| c5a.24xlarge     | $4.608/hr | 5.5 proofs/min    |

Before you begin, you will need to have the following software installed:
- `PostgreSQL`, [How to install PostgreSQL](https://www.postgresql.org/download/linux/ubuntu/)

You also need to install the following dependencies:
```shell
sudo apt-get update
sudo apt-get install libpq-dev libssl-dev pkg-config axel
curl -qL https://www.npmjs.com/install.sh | sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo install diesel_cli --no-default-features --features postgres

```
Note: For the first time, you need to set the password. Please refer to [psql.md](docs/psql.md) for the steps.
```bash
export DATABASE_URL=postgres://postgres:password@localhost/plasma
```
## Getting Started
### Clone the git repository and download the setup file:

```shell
git clone https://github.com/zkLinkProtocol/recover_state_server.git
cd recover_state_server/zklink_keys
axel -c https://universal-setup.ams3.digitaloceanspaces.com/setup_2%5E21.key
```

-----
### Configure the Environment Variables
There is a `.env.eg` file in the root path of our project, copy and rename it to `.env`. 
```shell
cp .env.e.g .env
```
**Must configure `RUNTIME_CONFIG_ZKLINK_HOME` and `DATABASE_URL` based on your environment.**

Explanation of .env Configuration Items: [env.md](env.md)

**NOTE**
Before "dunkerque," a link will be published here that will display contract and chain configurations. if test, only use default.

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
