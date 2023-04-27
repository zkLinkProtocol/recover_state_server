# Rename this example file to `.env` and place it in the current directory

# [rust]
# Configure the log level. For more information, refer to https://docs.rs/env_logger/0.9.1/env_logger/#enabling-logging
# For local development, set "sqlx::query=info" to display SQL output details
# Alternatively, set RUST_LOG="debug" for more detailed logs
# For testnet or mainnet, set "sqlx::query=error" to reduce SQLx log output
RUST_LOG="info,sqlx::query=error"

# [runtime]
# ZKLINK HOME path for storing data files
RUNTIME_CONFIG_ZKLINK_HOME="/home/user/zklink/recover_state_server"
# Directory for circuit keys
RUNTIME_CONFIG_KEY_DIR="zklink_keys"
# How many blocks we will process at once.
VIEW_BLOCK_STEP=1000

# [api]
API_CONFIG_SERVER_HTTP_PORT=8080
API_CONFIG_WORKERS_NUM=4
API_CONFIG_ENABLE_HTTP_CORS=true

# [database]
# Replace `USER_NAME` and `HOST` in the database URL with your PostgreSQL username
DATABASE_URL="postgres://postgres:password@localhost/plasma"
# Number of open connections to the database
DATABASE_POOL_SIZE=10

# Core application settings
# [prover.core]
# Timeout (in milliseconds) before considering a prover inactive
PROVER_CORE_GONE_TIMEOUT=60000
# Number of provers in the cluster when there are no pending jobs
PROVER_CORE_IDLE_PROVERS=1

# [chains]
# Chains that supported, the chain id is defined by zkLink
# We can add new chains, but can't drop an exist chain, that means
# We could set this option to "1,2" and then add a new chain(3)
# But we could not drop chain(1) or chain(2)
# NOTE, the max value of `CHAIN_IDS` must not be greater than `MAX_CHAIN_ID` defined in `params.rs`
CHAIN_IDS=1,2,3,4,6,7

# [chain_1.chain]
# Chain id defined by zkLink, must be equal to the placeholder of `CHAIN_{CHAIN_ID}_CHAIN_ID`
#CHAIN_1_CHAIN_ID=1
# Layer one chain type, for example, the chain type of Ethereum is EVM
#CHAIN_1_CHAIN_TYPE=EVM
# Gas token price symbol
#CHAIN_1_GAS_TOKEN=MATIC
# Whether sender should commit compressed block
# It must be keep same with the constant `ENABLE_COMMIT_COMPRESSED_BLOCK` defined in zkLink contract
#CHAIN_1_IS_COMMIT_COMPRESSED_BLOCKS=false

# [chain_1.contracts]
# The block number of contracts deployed
#CHAIN_1_CONTRACT_DEPLOYMENT_BLOCK=34850497
# The zkLink main contract address
#CHAIN_1_CONTRACT_ADDRESS="0x612962eB154aF944c79f3Eb3e1f4183f65DA2DD3"

# The zkLink contract deployed tx hash, used for recover data
#CHAIN_1_CONTRACT_GENESIS_TX_HASH="0x202e1683dd3a34ad70832713f901cd4452902cf540501732f87f808b90ba47d0"

# [chain_1.client]
# Chain id defined in layer one
#CHAIN_1_CLIENT_CHAIN_ID=80001
# RPC Server url of blockchain1.
#CHAIN_1_CLIENT_WEB3_URL="https://rpc.ankr.com/polygon_mumbai"
# The rpc service provider asked for a delay in the request because the number of requests was too frequent.
# It is configured according to the documentation of the rpc service
# The default configuration comes from the Infura docs(https://docs.infura.io/infura/networks/ethereum/how-to/avoid-rate-limiting).
# CHAIN_1_CLIENT_REQUEST_RATE_LIMIT_DELAY=30

CHAIN_1_CHAIN_ID=1
CHAIN_1_CHAIN_TYPE=EVM
CHAIN_1_GAS_TOKEN=MATIC
CHAIN_1_IS_COMMIT_COMPRESSED_BLOCKS=false
CHAIN_1_CONTRACT_DEPLOYMENT_BLOCK=34846518
CHAIN_1_CONTRACT_ADDRESS="0x04ab83F7DD4F8F376606eBCc4ED27d5B417E2EFD"
CHAIN_1_CONTRACT_GENESIS_TX_HASH="0xb9d9ecb261b30c05404b57b96609226ad225fa30a4f74fd60175758cbb6c737a"
CHAIN_1_CLIENT_CHAIN_ID=80001
CHAIN_1_CLIENT_WEB3_URL="https://rpc.ankr.com/polygon_mumbai"
CHAIN_1_CLIENT_REQUEST_RATE_LIMIT_DELAY=30

CHAIN_2_CHAIN_ID=2
CHAIN_2_CHAIN_TYPE=EVM
CHAIN_2_GAS_TOKEN=AVAX
CHAIN_2_IS_COMMIT_COMPRESSED_BLOCKS=false
CHAIN_2_CONTRACT_DEPLOYMENT_BLOCK=21260925
CHAIN_2_CONTRACT_ADDRESS="0x648c1e8758fF1469276e2AFC2BeAAB73F71a9b01"
CHAIN_2_CONTRACT_GENESIS_TX_HASH="0x8eed006cddcedb5015a363f0079dfd870ee67905842507116fc1fed49edd61b9"
CHAIN_2_CLIENT_CHAIN_ID=43113
CHAIN_2_CLIENT_WEB3_URL="https://rpc.ankr.com/avalanche_fuji"
CHAIN_2_CLIENT_REQUEST_RATE_LIMIT_DELAY=30

CHAIN_3_CHAIN_ID=3
CHAIN_3_CHAIN_TYPE=EVM
CHAIN_3_GAS_TOKEN=BNB
CHAIN_3_IS_COMMIT_COMPRESSED_BLOCKS=false
CHAIN_3_CONTRACT_DEPLOYMENT_BLOCK=29270728
CHAIN_3_CONTRACT_ADDRESS="0x8119A046288Aae708b0DC3Eb852D6e857609F915"
CHAIN_3_CONTRACT_GENESIS_TX_HASH="0x6f8dd23ec3944729f45fe334ccc77b5a76abd67b1cc49fc38a4dfad69deb2cc4"
CHAIN_3_CLIENT_CHAIN_ID=97
CHAIN_3_CLIENT_WEB3_URL="https://data-seed-prebsc-1-s3.binance.org:8545"
CHAIN_3_CLIENT_REQUEST_RATE_LIMIT_DELAY=30

CHAIN_4_CHAIN_ID=4
CHAIN_4_CHAIN_TYPE=EVM
CHAIN_4_GAS_TOKEN=ETH
CHAIN_4_IS_COMMIT_COMPRESSED_BLOCKS=false
CHAIN_4_CONTRACT_DEPLOYMENT_BLOCK=8894313
CHAIN_4_CONTRACT_ADDRESS="0x790869FF44a89eFAb1BE81Cb2233143B4dd0064B"
CHAIN_4_CONTRACT_GENESIS_TX_HASH="0x46e8eddfe05b50b1640dff2b78416608ff17a53254e62e35d1cb7d3f6ccdb75b"
CHAIN_4_CLIENT_CHAIN_ID=5
CHAIN_4_CLIENT_WEB3_URL="https://rpc.ankr.com/eth_goerli"
CHAIN_4_CLIENT_REQUEST_RATE_LIMIT_DELAY=30

CHAIN_5_CHAIN_ID=5
CHAIN_5_CHAIN_TYPE=EVM
CHAIN_5_GAS_TOKEN=ETH
CHAIN_5_IS_COMMIT_COMPRESSED_BLOCKS=true
CHAIN_5_CONTRACT_DEPLOYMENT_BLOCK=0
CHAIN_5_CONTRACT_ADDRESS="0x0000000000000000000000000000000000000000000000000000000000000000"
CHAIN_5_CONTRACT_GENESIS_TX_HASH="0x0000000000000000000000000000000000000000000000000000000000000000"
CHAIN_5_CLIENT_CHAIN_ID=280
CHAIN_5_CLIENT_WEB3_URL="https://testnet.era.zksync.dev"
CHAIN_5_CLIENT_REQUEST_RATE_LIMIT_DELAY=30

CHAIN_6_CHAIN_ID=6
CHAIN_6_CHAIN_TYPE=EVM
CHAIN_6_GAS_TOKEN=ETH
CHAIN_6_IS_COMMIT_COMPRESSED_BLOCKS=false
CHAIN_6_CONTRACT_DEPLOYMENT_BLOCK=1645703
CHAIN_6_CONTRACT_ADDRESS="0x43D2F45cB48e1ddC6871B6B729BAC95769e7cAd9"
CHAIN_6_CONTRACT_GENESIS_TX_HASH="0x2e73d413d08bbfe554974a8250727e47208f88a461243b99a020d5445a333a09"
CHAIN_6_CLIENT_CHAIN_ID=534353
CHAIN_6_CLIENT_WEB3_URL="https://scroll-testnet.blockpi.network/v1/rpc/public"
CHAIN_6_CLIENT_REQUEST_RATE_LIMIT_DELAY=30

CHAIN_7_CHAIN_ID=7
CHAIN_7_CHAIN_TYPE=EVM
CHAIN_7_GAS_TOKEN=ETH
CHAIN_7_IS_COMMIT_COMPRESSED_BLOCKS=false
CHAIN_7_CONTRACT_DEPLOYMENT_BLOCK=562013
CHAIN_7_CONTRACT_ADDRESS="0x17f05b5C562BAd0949A8f533536aFD2cB44a4A62"
CHAIN_7_CONTRACT_GENESIS_TX_HASH="0x7cc808aa18859663cf21377e0ead26db3e04d929fcc995c5727a25eddd28937b"
CHAIN_7_CLIENT_CHAIN_ID=59140
CHAIN_7_CLIENT_WEB3_URL="https://rpc.goerli.linea.build"
CHAIN_7_CLIENT_REQUEST_RATE_LIMIT_DELAY=30
