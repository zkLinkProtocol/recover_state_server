# zkLink Exodus Model
This repository contains the Server, Prover and React App for zkLink Exodus Model.

## Table of Contents
- [Prerequisites](#prerequisites)
- [Getting Started](#getting-started)
  - [Clone the git repository and download the setup file](#clone-the-git-repository-and-download-the-setup-file)
  - [Configure the Environment Variables](#configure-the-environment-variables)
  - [Starting the recovery program, prover program, and web service server](#starting-the-recovery-program-prover-program-and-web-service-server)
  - [Stopping recovery program, Prover program, and web service server](#stopping-recovery-program-prover-program-and-web-service-server)
  - [Cleaning Up All Exodus Data](#cleaning-up-all-exodus-data)
- [License](#license)


## Prerequisites
We recommend using the Ubuntu OS, and below are three recommended configurations.

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
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.3/install.sh | bash
nvm install v16.20
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
cd recover_state_server
axel -c https://universal-setup.ams3.digitaloceanspaces.com/setup_2%5E23.key -o ./zklink_keys
```

-----
### Configure the Environment Variables
There is a `.env.e.g` file in the root path of our project, copy and rename it to `.env`. 
```shell
cp .env.e.g .env
```
**Must configure `RUNTIME_CONFIG_ZKLINK_HOME` and `DATABASE_URL` based on your environment.**

Explanation of .env Configuration Items: [env.md](env.md)

**NOTE**
Before "dunkerque," a link will be published here that will display contract and chain configurations. if test, only use default.

-----

### Starting the recovery program, prover program, and web service server
```shell
export PORT=80  # The access port for the frontend page.

./exodus.sh start
```
This command may take several hours to complete.

If you want to monitor the state recovery process, please run the following command:
```shell
tail -f log/recover_state.log
```
The recovery program will close automatically when synchronization is complete.

### Stopping recovery program, Prover program, and web service server
```shell
./exodus.sh stop
```

### Cleaning Up All Exodus Data
```shell
./exodus.sh clean
```

## License
This project is licensed under the MIT License - see the `LICENSE` file for details.
