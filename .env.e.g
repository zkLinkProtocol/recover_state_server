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

# [api]
API_CONFIG_SERVER_HTTP_PORT=8080
API_CONFIG_WORKERS_NUM=4
API_CONFIG_ENABLE_HTTP_CORS=true

# [database]
# Replace `USER_NAME` and `HOST` in the database URL with your PostgreSQL username
DATABASE_URL="postgres://postgres:postgres@localhost/plasma"
# Number of open connections to the database
DATABASE_POOL_SIZE=10

# [chains]
# Supported chains with zkLink-defined chain IDs
# To add new chains, include their IDs (e.g., "1,2" to add chain 3)
# Note: Existing chains cannot be removed
# The maximum value of `CHAIN_IDS` must not exceed `MAX_CHAIN_ID` defined in `params.rs`
CHAIN_IDS=1,2

# [chain_1.chain]
# zkLink-defined chain ID, must match the `CHAIN_{CHAIN_ID}_CHAIN_ID` placeholder
CHAIN_1_CHAIN_ID=1
# Layer one chain type, e.g., Ethereum's chain type is EVM
CHAIN_1_CHAIN_TYPE=EVM
# Gas token price symbol
CHAIN_1_GAS_TOKEN=MATIC
# Indicates whether the sender should commit compressed blocks
# This value must match the `ENABLE_COMMIT_COMPRESSED_BLOCK` constant defined in the zkLink contract
CHAIN_1_IS_COMMIT_COMPRESSED_BLOCKS=true

# [chain_1.CONTRACT]
# Deployment block number of the CONTRACT
CHAIN_1_CONTRACT_DEPLOYMENT_BLOCK=33377564
# Address of the zkLink main contract
CHAIN_1_CONTRACT_ADDRESS="0x517aa9dec0E297B744aC7Ac8ddd8B127c1993055"
# Transaction hash of the deployed zkLink contract, used for data recovery
CHAIN_1_CONTRACT_GENESIS_TX_HASH="0x5c576039ffefce307ffbc5556899ee0772efcf2046051cc4fe9ca633987061ca"

# [chain_1.client]
# Chain ID defined in layer one
CHAIN_1_CLIENT_CHAIN_ID=80001
# RPC server URL for blockchain1
CHAIN_1_CLIENT_WEB3_URL="https://rpc.ankr.com/polygon_mumbai"
# Configure the delay (in milliseconds) for RPC requests when the service provider limits the request rate
# Refer to the RPC service provider's documentation for configuration details
# Default settings are from the Infura docs (https://docs.infura.io/infura/networks/ethereum/how-to/avoid-rate-limiting)
CHAIN_1_CLIENT_REQUEST_RATE_LIMIT_DELAY=30

# [chain_2.chain]
# zkLink-defined chain ID, must match the `CHAIN_{CHAIN_ID}_CHAIN_ID` placeholder
CHAIN_2_CHAIN_ID=2
# Layer one chain type, e.g., Ethereum's chain type is EVM
CHAIN_2_CHAIN_TYPE=EVM
# Gas token price symbol
CHAIN_2_GAS_TOKEN=AVAX
# Indicates whether the sender should commit compressed blocks
# This value must match the `ENABLE_COMMIT_COMPRESSED_BLOCK` constant defined in the zkLink contract
CHAIN_2_IS_COMMIT_COMPRESSED_BLOCKS=false

# [chain_2.CONTRACT]
# Deployment block number of the CONTRACT
CHAIN_2_CONTRACT_DEPLOYMENT_BLOCK=20072376
# Address of the zkLink main contract
CHAIN_2_CONTRACT_ADDRESS="0x331a96b91F35051706680d96251931E26f4ba58A"
# Transaction hash of the deployed zkLink contract, used for data recovery
CHAIN_2_CONTRACT_GENESIS_TX_HASH="0xce20f9d8eeea9b9eb378d1ce4960c4aa89701f4ed0ae24638c57984f8af3f6ef"

# [chain_2.client]
# Chain ID defined in layer one
CHAIN_2_CLIENT_CHAIN_ID=43113
# RPC server URL for blockchain2
CHAIN_2_CLIENT_WEB3_URL="https://rpc.ankr.com/avalanche_fuji"
# Configure the delay (in milliseconds) for RPC requests when the service provider limits the request rate
# Refer to the RPC service provider's documentation for configuration details
# Default settings are from the Infura docs (https://docs.infura.io/infura/networks/ethereum/how-to/avoid-rate-limiting)
CHAIN_2_CLIENT_REQUEST_RATE_LIMIT_DELAY=30

# Core application settings
# [prover.core]
# Timeout (in milliseconds) before considering a prover inactive
PROVER_CORE_GONE_TIMEOUT=60000
# Number of provers in the cluster when there are no pending jobs
PROVER_CORE_IDLE_PROVERS=1
