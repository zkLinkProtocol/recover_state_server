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
| c5a.4xlarge      | $0.768/hr | 0.3 proofs/min    |
| c5a.12xlarge     | $2.304/hr | 1 proofs/min      |
| c5a.24xlarge     | $4.608/hr | 2 proofs/min    |

Before you begin, you will need to have the following software installed:
- `PostgreSQL`, [How to install PostgreSQL](https://www.postgresql.org/download/linux/ubuntu/)

```bash

sudo sh -c 'echo "deb http://apt.postgresql.org/pub/repos/apt $(lsb_release -cs)-pgdg main" > /etc/apt/sources.list.d/pgdg.list'

wget --quiet -O - https://www.postgresql.org/media/keys/ACCC4CF8.asc | sudo apt-key add -

sudo apt-get update

sudo apt-get -y install postgresql

````
You also need to install the following dependencies:
```shell

sudo apt-get install libpq-dev libssl-dev pkg-config axel

curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.3/install.sh | bash

export NVM_DIR="$HOME/.nvm"

[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"  # This loads nvm

nvm install v16.20

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

sudo apt install build-essential

cargo install diesel_cli --no-default-features --features postgres

```
Note: For the first time, you need to set the psql password.

```bash
sudo su - postgres

#connect to postgresql
psql

#modify password
\password
```

```bash
echo 'export DATABASE_URL=postgres://postgres:password@localhost/plasma' >> ~/.bashrc

source ~/.bashrc
```

To accommodate a high volume of requests on the Web service, append the following line to the `/etc/security/limits.conf` file:

```
* soft nofile 1048576
```

```bash
# To ensure that the changes take effect, you should log out and log back in to the current shell session.
exit 
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

# Must configure `RUNTIME_CONFIG_ZKLINK_HOME` and `DATABASE_URL` based on your environment
sed -i "s|/home/user/zklink/recover_state_server|$(pwd)|g" .env

sed -i "s|postgres://postgres:password@localhost/plasma|${DATABASE_URL}|g" .env

```

Explanation of .env Configuration Items: [env.md](docs/env.md)


We recommend displaying your `NAME` and `LOGO` on the recovery page, as it can help with the branding of the recovery node operator. Please refer to the [exodus-interface/README.md](exodus-interface/README.md) for more information.


-----

### Starting the recovery program, prover program, and web service server
```shell
export PORT=8081  # The access port for the frontend page.

# Forward traffic from port 80 to port 8081
sudo iptables -t nat -A PREROUTING -p tcp --dport 80 -j REDIRECT --to-port 8081

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
