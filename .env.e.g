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
VIEW_BLOCK_STEP=100

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
CHAIN_1_CHAIN_ID=1
# Layer one chain type, for example, the chain type of Ethereum is EVM
CHAIN_1_CHAIN_TYPE=EVM
# Gas token price symbol
CHAIN_1_GAS_TOKEN=MATIC
# Whether sender should commit compressed block
# It must be keep same with the constant `ENABLE_COMMIT_COMPRESSED_BLOCK` defined in zkLink contract
CHAIN_1_IS_COMMIT_COMPRESSED_BLOCKS=false

# [chain_1.contracts]
# The block number of contracts deployed
CHAIN_1_CONTRACT_DEPLOYMENT_BLOCK=34887307
# The zkLink main contract address
CHAIN_1_CONTRACT_ADDRESS="0xAbbf03f6baC3E06c7C7fA6dec0f865bfb2aCf8b2"

# The zkLink contract deployed tx hash, used for recover data
CHAIN_1_CONTRACT_GENESIS_TX_HASH="0x80b1a303553ce8763292724166b5612759c1c44dbd83d845ae6ee334b6eb6117"

# [chain_1.client]
# Chain id defined in layer one
CHAIN_1_CLIENT_CHAIN_ID=80001
# RPC Server url of blockchain1.
CHAIN_1_CLIENT_WEB3_URL="https://rpc.ankr.com/polygon_mumbai"
# The rpc service provider asked for a delay in the request because the number of requests was too frequent.
# It is configured according to the documentation of the rpc service
# The default configuration comes from the Infura docs(https://docs.infura.io/infura/networks/ethereum/how-to/avoid-rate-limiting).
CHAIN_1_CLIENT_REQUEST_RATE_LIMIT_DELAY=30

# [chain_2.chain]
# Chain id defined by zkLink, must be equal to the placeholder of `CHAIN_{CHAIN_ID}_CHAIN_ID`
CHAIN_2_CHAIN_ID=2
# Layer one chain type, for example, the chain type of Ethereum is EVM
CHAIN_2_CHAIN_TYPE=EVM
# Gas token price symbol
CHAIN_2_GAS_TOKEN=AVAX
# Whether sender should commit compressed block
# It must be keep same with the constant `ENABLE_COMMIT_COMPRESSED_BLOCK` defined in zkLink contract
CHAIN_2_IS_COMMIT_COMPRESSED_BLOCKS=false

# [chain_2.contracts]
# The block number of contracts deployed
CHAIN_2_CONTRACT_DEPLOYMENT_BLOCK=21292058
# The zkLink main contract address
CHAIN_2_CONTRACT_ADDRESS="0x4196e73177AFfD8BE3095aaD7F88CDA20994bBfF"
# The zkLink contract deployed tx hash, used for recover data
CHAIN_2_CONTRACT_GENESIS_TX_HASH="0x5ae495454766108e596d2bb083f3668c97c0640e7a6808e9eddfa1ba71f7afda"

# [chain_2.client]
# Chain id defined in layer one
CHAIN_2_CLIENT_CHAIN_ID=43113
# RPC Server url of blockchain1.
CHAIN_2_CLIENT_WEB3_URL="https://rpc.ankr.com/avalanche_fuji"
# The rpc service provider asked for a delay in the request because the number of requests was too frequent.
# It is configured according to the documentation of the rpc service
# The default configuration comes from the Infura docs(https://docs.infura.io/infura/networks/ethereum/how-to/avoid-rate-limiting).
CHAIN_2_CLIENT_REQUEST_RATE_LIMIT_DELAY=30

# [chain_3.chain]
# Chain id defined by zkLink, must be equal to the placeholder of `CHAIN_{CHAIN_ID}_CHAIN_ID`
CHAIN_3_CHAIN_ID=3
# Layer one chain type, for example, the chain type of Ethereum is EVM
CHAIN_3_CHAIN_TYPE=EVM
# Gas token price symbol
CHAIN_3_GAS_TOKEN=BNB
# Whether sender should commit compressed block
# It must be keep same with the constant `ENABLE_COMMIT_COMPRESSED_BLOCK` defined in zkLink contract
CHAIN_3_IS_COMMIT_COMPRESSED_BLOCKS=false

# [chain_3.contracts]
# The block number of contracts deployed
CHAIN_3_CONTRACT_DEPLOYMENT_BLOCK=29299507
# The zkLink main contract address
CHAIN_3_CONTRACT_ADDRESS="0xb6CBd075C1f6665DfAF4ca2B68376D7653c641D7"

# The zkLink contract deployed tx hash, used for recover data
CHAIN_3_CONTRACT_GENESIS_TX_HASH="0x9b9c90691734686c2ed9bceb25acbb86d1d1c0d7c93114bfee16c21d76090931"

# [chain_3.client]
# Chain id defined in layer one
CHAIN_3_CLIENT_CHAIN_ID=97
# RPC Server url of blockchain1.
CHAIN_3_CLIENT_WEB3_URL="https://data-seed-prebsc-1-s3.binance.org:8545"
# The rpc service provider asked for a delay in the request because the number of requests was too frequent.
# It is configured according to the documentation of the rpc service
# The default configuration comes from the Infura docs(https://docs.infura.io/infura/networks/ethereum/how-to/avoid-rate-limiting).
CHAIN_3_CLIENT_REQUEST_RATE_LIMIT_DELAY=30

# [chain_4.chain]
# Chain id defined by zkLink, must be equal to the placeholder of `CHAIN_{CHAIN_ID}_CHAIN_ID`
CHAIN_4_CHAIN_ID=4
# Layer one chain type, for example, the chain type of Ethereum is EVM
CHAIN_4_CHAIN_TYPE=EVM
# Gas token price symbol
CHAIN_4_GAS_TOKEN=ETH
# Whether sender should commit compressed block
# It must be keep same with the constant `ENABLE_COMMIT_COMPRESSED_BLOCK` defined in zkLink contract
CHAIN_4_IS_COMMIT_COMPRESSED_BLOCKS=false

# [chain_4.contracts]
# The block number of contracts deployed
CHAIN_4_CONTRACT_DEPLOYMENT_BLOCK=8899983
# The zkLink main contract address
CHAIN_4_CONTRACT_ADDRESS="0xCE9505eEb2240340B6a95672FA07D83752E986DE"

# The zkLink contract deployed tx hash, used for recover data
CHAIN_4_CONTRACT_GENESIS_TX_HASH="0xcd8ea8cef283465a2703d0e3b64077b2fca6aaacf5c67defda6c787e5dca342c"

# [chain_4.client]
# Chain id defined in layer one
CHAIN_4_CLIENT_CHAIN_ID=5
# RPC Server url of blockchain1.
CHAIN_4_CLIENT_WEB3_URL="https://rpc.ankr.com/eth_goerli"
# The rpc service provider asked for a delay in the request because the number of requests was too frequent.
# It is configured according to the documentation of the rpc service
# The default configuration comes from the Infura docs(https://docs.infura.io/infura/networks/ethereum/how-to/avoid-rate-limiting).
CHAIN_4_CLIENT_REQUEST_RATE_LIMIT_DELAY=30

# [chain_5.chain]
# Chain id defined by zkLink, must be equal to the placeholder of `CHAIN_{CHAIN_ID}_CHAIN_ID`
CHAIN_5_CHAIN_ID=5
# Layer one chain type, for example, the chain type of Ethereum is EVM
CHAIN_5_CHAIN_TYPE=EVM
# Gas token price symbol
CHAIN_5_GAS_TOKEN=ETH
# Whether sender should commit compressed block
# It must be keep same with the constant `ENABLE_COMMIT_COMPRESSED_BLOCK` defined in zkLink contract
CHAIN_5_IS_COMMIT_COMPRESSED_BLOCKS=true

# [chain_5.contracts]
# The block number of contracts deployed
CHAIN_5_CONTRACT_DEPLOYMENT_BLOCK=0
# The zkLink main contract address
CHAIN_5_CONTRACT_ADDRESS="0x0000000000000000000000000000000000000000000000000000000000000000"

# The zkLink contract deployed tx hash, used for recover data
CHAIN_5_CONTRACT_GENESIS_TX_HASH="0x0000000000000000000000000000000000000000000000000000000000000000"

# [chain_5.client]
# Chain id defined in layer one
CHAIN_5_CLIENT_CHAIN_ID=280
# RPC Server url of blockchain1.
CHAIN_5_CLIENT_WEB3_URL="https://testnet.era.zksync.dev"
# The rpc service provider asked for a delay in the request because the number of requests was too frequent.
# It is configured according to the documentation of the rpc service
# The default configuration comes from the Infura docs(https://docs.infura.io/infura/networks/ethereum/how-to/avoid-rate-limiting).
CHAIN_5_CLIENT_REQUEST_RATE_LIMIT_DELAY=30

# [chain_6.chain]
# Chain id defined by zkLink, must be equal to the placeholder of `CHAIN_{CHAIN_ID}_CHAIN_ID`
CHAIN_6_CHAIN_ID=6
# Layer one chain type, for example, the chain type of Ethereum is EVM
CHAIN_6_CHAIN_TYPE=EVM
# Gas token price symbol
CHAIN_6_GAS_TOKEN=ETH
# Whether sender should commit compressed block
# It must be keep same with the constant `ENABLE_COMMIT_COMPRESSED_BLOCK` defined in zkLink contract
CHAIN_6_IS_COMMIT_COMPRESSED_BLOCKS=false

# [chain_6.contracts]
# The block number of contracts deployed
CHAIN_6_CONTRACT_DEPLOYMENT_BLOCK=1674567
# The zkLink main contract address
CHAIN_6_CONTRACT_ADDRESS="0xBcb2513B760CAa7E42117B7b72461da12d88cD1f"

# The zkLink contract deployed tx hash, used for recover data
CHAIN_6_CONTRACT_GENESIS_TX_HASH="0xb8bda016af3e2ea8a291d07dc1590e08927ecf93f935cd1222f364aa98fed030"

# [chain_6.client]
# Chain id defined in layer one
CHAIN_6_CLIENT_CHAIN_ID=534353
# RPC Server url of blockchain1.
CHAIN_6_CLIENT_WEB3_URL="https://scroll-testnet.blockpi.network/v1/rpc/public"
# The rpc service provider asked for a delay in the request because the number of requests was too frequent.
# It is configured according to the documentation of the rpc service
# The default configuration comes from the Infura docs(https://docs.infura.io/infura/networks/ethereum/how-to/avoid-rate-limiting).
CHAIN_6_CLIENT_REQUEST_RATE_LIMIT_DELAY=30

# [chain_7.chain]
# Chain id defined by zkLink, must be equal to the placeholder of `CHAIN_{CHAIN_ID}_CHAIN_ID`
CHAIN_7_CHAIN_ID=7
# Layer one chain type, for example, the chain type of Ethereum is EVM
CHAIN_7_CHAIN_TYPE=EVM
# Gas token price symbol
CHAIN_7_GAS_TOKEN=ETH
# Whether sender should commit compressed block
# It must be keep same with the constant `ENABLE_COMMIT_COMPRESSED_BLOCK` defined in zkLink contract
CHAIN_7_IS_COMMIT_COMPRESSED_BLOCKS=false

# [chain_7.contracts]
# The block number of contracts deployed
CHAIN_7_CONTRACT_DEPLOYMENT_BLOCK=569230
# The zkLink main contract address
CHAIN_7_CONTRACT_ADDRESS="0xb0BBfEc0302032bec5d6fDF8050e4f56Bc3F42dB"

# The zkLink contract deployed tx hash, used for recover data
CHAIN_7_CONTRACT_GENESIS_TX_HASH="0x27190dfc54b9a8b5457743d559f8392fe45e63065a71473e3f6c36c549f96d42"

# [chain_7.client]
# Chain id defined in layer one
CHAIN_7_CLIENT_CHAIN_ID=59140
# RPC Server url of blockchain1.
CHAIN_7_CLIENT_WEB3_URL="https://rpc.goerli.linea.build"
# The rpc service provider asked for a delay in the request because the number of requests was too frequent.
# It is configured according to the documentation of the rpc service
# The default configuration comes from the Infura docs(https://docs.infura.io/infura/networks/ethereum/how-to/avoid-rate-limiting).
CHAIN_7_CLIENT_REQUEST_RATE_LIMIT_DELAY=30
