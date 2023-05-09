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
DATABASE_URL="postgres://postgres:password@localhost/plasma"
# Number of open connections to the database
DATABASE_POOL_SIZE=100

# Core application settings
# [prover.core]
# Timeout (in milliseconds) before considering a prover inactive
PROVER_CORE_GONE_TIMEOUT=60000
# Number of provers in the cluster when there are no pending jobs
PROVER_CORE_IDLE_PROVERS=1

# [special]
# Periodically(in minutes) clean up blacklisted users (to prevent users from requesting too many proof tasks)
CLEAN_INTERVAL=180

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
CHAIN_1_CONTRACT_DEPLOYMENT_BLOCK=34920104
# The zkLink main contract address
CHAIN_1_CONTRACT_ADDRESS="0xd5a67aE094D26451C5CE592798C9CaDE55f968aa"

# The zkLink contract deployed tx hash, used for recover data
CHAIN_1_CONTRACT_GENESIS_TX_HASH="0x55df09af604606e03193f8483bcbfe72aa351e437388f1a217a8e110f07a9050"

# [chain_1.client]
# Chain id defined in layer one
CHAIN_1_CLIENT_CHAIN_ID=80001
# RPC Server url of blockchain1.
CHAIN_1_CLIENT_WEB3_URL="https://rpc.ankr.com/polygon_mumbai"
# The step of every view blocks.
CHAIN_1_CLIENT_VIEW_BLOCK_STEP=3000
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
CHAIN_2_CONTRACT_DEPLOYMENT_BLOCK=21316425
# The zkLink main contract address
CHAIN_2_CONTRACT_ADDRESS="0x7a185Fa2CC782639bCEeb28ecD0cD85b8709EC98"
# The zkLink contract deployed tx hash, used for recover data
CHAIN_2_CONTRACT_GENESIS_TX_HASH="0xa5e208014e89174bffed377d04b1a9e24190616598fd2bfe08c4228ecc602b6d"

# [chain_2.client]
# Chain id defined in layer one
CHAIN_2_CLIENT_CHAIN_ID=43113
# RPC Server url of blockchain1.
CHAIN_2_CLIENT_WEB3_URL="https://rpc.ankr.com/avalanche_fuji"
# The step of every view blocks.
CHAIN_2_CLIENT_VIEW_BLOCK_STEP=3000
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
CHAIN_3_CONTRACT_DEPLOYMENT_BLOCK=29322741
# The zkLink main contract address
CHAIN_3_CONTRACT_ADDRESS="0x15ee6c6360f62db16250B84A2efDA48f001740E8"

# The zkLink contract deployed tx hash, used for recover data
CHAIN_3_CONTRACT_GENESIS_TX_HASH="0x547d16698a2de3d63def0683ee7cddb091744a9993e40cbd6877d53af2473a2d"

# [chain_3.client]
# Chain id defined in layer one
CHAIN_3_CLIENT_CHAIN_ID=97
# RPC Server url of blockchain1.
CHAIN_3_CLIENT_WEB3_URL="https://rpc-forward.zk.link/rpc?chain=bsc"
# The step of every view blocks.
CHAIN_3_CLIENT_VIEW_BLOCK_STEP=2000
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
CHAIN_4_CONTRACT_DEPLOYMENT_BLOCK=8904500
# The zkLink main contract address
CHAIN_4_CONTRACT_ADDRESS="0x4d116306C418010F85d6905457239349914bF1Cd"

# The zkLink contract deployed tx hash, used for recover data
CHAIN_4_CONTRACT_GENESIS_TX_HASH="0x95004489ce3bebeac824d20f48da267292c5725aaef543c338f5f4d10e61075a"

# [chain_4.client]
# Chain id defined in layer one
CHAIN_4_CLIENT_CHAIN_ID=5
# RPC Server url of blockchain1.
CHAIN_4_CLIENT_WEB3_URL="https://rpc.ankr.com/eth_goerli"
# The step of every view blocks.
CHAIN_4_CLIENT_VIEW_BLOCK_STEP=2000
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
CHAIN_5_IS_COMMIT_COMPRESSED_BLOCKS=false

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
# The step of every view blocks.
CHAIN_5_CLIENT_VIEW_BLOCK_STEP=2000
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
CHAIN_6_CONTRACT_DEPLOYMENT_BLOCK=1697790
# The zkLink main contract address
CHAIN_6_CONTRACT_ADDRESS="0xcC85Ae89DC053e34a58f04e88571644F41A0e5c0"

# The zkLink contract deployed tx hash, used for recover data
CHAIN_6_CONTRACT_GENESIS_TX_HASH="0x5d240af705735ef317990ffa610e8803358f97fe3a161bb5719d5b929c19af63"

# [chain_6.client]
# Chain id defined in layer one
CHAIN_6_CLIENT_CHAIN_ID=534353
# RPC Server url of blockchain1.
CHAIN_6_CLIENT_WEB3_URL="https://rpc-forward.zk.link/rpc?chain=scroll"
# The step of every view blocks.
CHAIN_6_CLIENT_VIEW_BLOCK_STEP=1000
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
CHAIN_7_CONTRACT_DEPLOYMENT_BLOCK=575035
# The zkLink main contract address
CHAIN_7_CONTRACT_ADDRESS="0xc04A47344C362b6a4DD1E7b7Fd080ac6ABA36C95"

# The zkLink contract deployed tx hash, used for recover data
CHAIN_7_CONTRACT_GENESIS_TX_HASH="0xe84d641b0d9baff69fa3e3d5046f2a69e7bd965213c71c714d2a6abcbc41af63"

# [chain_7.client]
# Chain id defined in layer one
CHAIN_7_CLIENT_CHAIN_ID=59140
# RPC Server url of blockchain1.
CHAIN_7_CLIENT_WEB3_URL="https://rpc-forward.zk.link/rpc?chain=linea"
# The step of every view blocks.
CHAIN_7_CLIENT_VIEW_BLOCK_STEP=2000
# The rpc service provider asked for a delay in the request because the number of requests was too frequent.
# It is configured according to the documentation of the rpc service
# The default configuration comes from the Infura docs(https://docs.infura.io/infura/networks/ethereum/how-to/avoid-rate-limiting).
CHAIN_7_CLIENT_REQUEST_RATE_LIMIT_DELAY=30
