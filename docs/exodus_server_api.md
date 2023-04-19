# Exodus Server

- [Basic Structure](#Basic-Structure)
  - [Error Code](#Error-Code)
- [API](#API)
    - [contracts](#contracts)
    - [tokens(after completed state)](#tokens)
    - [recover_progress](#recover_progress)
    - [running_max_task_id(after completed state)](#running_max_task_id)
    - [get_token(after completed state)](#get_token)
    - [get_stored_block_info(after completed state)](#get_stored_block_info)
    - [get_balances(after completed state)](#get_balances)
    - [get_unprocessed_priority_ops(after completed state)](#get_unprocessed_priority_ops)
    - [get_proof_task_id(after completed state)](#get_proof_task_id)
    - [get_proof_by_info(after completed state)](#get_proof_by_info)
    - [get_proofs_by_id(after completed state)](#get_proofs_by_id)
    - [generate_proof_task_by_info(after completed state)](#generate_proof_task_by_info)
    - [generate_proof_tasks_by_token(after completed state)](#generate_proof_tasks_by_token)

## Basic Structure
### Error Code and message
```rust
enum ExodusError {
    Ok = 0,
    ProofTaskAlreadyExists = 50,
    ProofGenerating = 51,
    ProofCompleted = 52,
    NonBalance = 60,
    RecoverStateUnfinished = 70,

    TokenNotExist = 101,
    AccountNotExist = 102,
    ChainNotExist = 103,
    ExitProofTaskNotExist = 104,

    InvalidL1L2Token = 201,

    InternalErr=500
}

impl ToString for ExodusError {
    fn to_string(&self) -> String {
        match self {
            // Normal response
            ExodusError::Ok => "Ok",
            ExodusError::ProofTaskAlreadyExists => "The proof Task already exists",
            ExodusError::ProofGenerating => "The proof task is running",
            ExodusError::ProofCompleted => "The task has been completed",
            ExodusError::NonBalance => "The token of the account is no balance",
            ExodusError::RecoverStateUnfinished => "Recovering state is unfinished",

            // Error response
            // Not exist info
            ExodusError::TokenNotExist => "The token not exist",
            ExodusError::AccountNotExist => "The account not exist",
            ExodusError::ChainNotExist => "The chain not exist",
            ExodusError::ExitProofTaskNotExist => "The exit proof task not exist",

            // Invalid parameters
            ExodusError::InvalidL1L2Token => "The relationship between l1 token and l2 token is incorrect",

            // Internal error,
            ExodusError::InternalErr => "Exodus server internal error",
        }.to_string()
    }
}
```
### StoredBlockInfo
| field                           | type        | description                                                             |
|---------------------------------|-------------|-------------------------------------------------------------------------|
| block_number                    | BlockNumber | Rollup block number                                                     |
| priority_operations             | u64         | Number of priority operations processed                                 |
| pending_onchain_operations_hash | H256        | Hash of all operations that must be processed after verify              |
| timestamp                       | U256        | Rollup block timestamp, have the same format as Ethereum block constant |
| state_hash                      | H256        | Root hash of the rollup state                                           |
| commitment                      | H256        | Verified input for the ZkLink circuit                                   |
| sync_hash                       | H256        | for cross chain block verify                                            |
```rust
struct StoredBlockInfo {
    block_number: BlockNumber, // Rollup block number
    priority_operations: u64, // Number of priority operations processed
    pending_onchain_operations_hash: H256, // Hash of all operations that must be processed after verify
    timestamp: U256, // Rollup block timestamp, have the same format as Ethereum block constant
    state_hash: H256, // Root hash of the rollup state
    commitment: H256, // Verified input for the ZkLink circuit
    sync_hash: H256 // Used for cross chain block verify
}
```
### UnprocessedPriorityOp
```rust
#[derive(Debug, Serialize, Deserialize,Clone)]
  pub struct UnprocessedPriorityOp {
  pub(crate) serial_id: SerialId,
  pub(crate) pub_data: PublicData
}

#[derive(Debug, Serialize, Deserialize,Clone)]
pub enum PublicData{
  Deposit(DepositData),
  FullExit
}

#[derive(Debug, Serialize, Deserialize,Clone)]
pub struct DepositData{
  chain_id: ChainId,
  sub_account_id: SubAccountId,
  l2_target_token_id: TokenId,
  l1_source_token_id: TokenId,
  #[serde(with = "BigUintSerdeAsRadix10Str")]
  amount: BigUint,
  owner: ZkLinkAddress,
}
```
### TokenInfo
| field     | type                            | description                                                 |
|-----------|---------------------------------|-------------------------------------------------------------|
| token_id  | TokenId                         | the id of token                                             |
| addresses | HashMap<ChainId, ZkLinkAddress> | The token is at the contract address of the different chain |
```rust
struct TokenInfo {
  token_id: TokenId,
  addresses: HashMap<ChainId, ZkLinkAddress>,
}
```
### ExitProofData
| field      | type                    | description                                          |
|------------|-------------------------|------------------------------------------------------|
| exit_info  | [ExitInfo](#ExitInfo)   | the info of exodus exit                              |
| proof_info | [ProofInfo](#ProofInfo) | the proof context of exodus exit, may be null        |
```rust
struct ExitProofData {
    exit_info: ExitInfo,
    amount: Option<BigUintSerdeWrapper>,
    proof: Option<EncodedSingleProof>,
}
```
### ExitInfo
| field           | type          | description                            |
|-----------------|---------------|----------------------------------------|
| chain_id        | u8            | the target chain id of exodus exit     |
| account_address | ZkLinkAddress | the address of exodus exit             |
| account_id      | u32           | the account_id of exodus exit          |
| sub_account_id  | u8            | the sub_account_id of exodus exit      |
| l1_target_token | u32           | the layer1 target token of exodus exit |
| l2_source_token | u32           | the layer2 source token of exodus exit |
```rust
struct ExitInfo {
    chain_id: ChainId, // u8
    account_address: ZkLinkAddress, // 20 bytes or 32 bytes
    account_id: AccountId, // u32
    sub_account_id: SubAccountId, // u8
    l1_target_token: TokenId, // u32
    l2_source_token: TokenId, // u32
}
```
### ProofInfo
| field  | type                        | description                 |
|--------|-----------------------------|-----------------------------|
| id     | ProofId(u64)                | the id of proof task        |
| amount | Option<BigUintSerdeWrapper> | the amount of exodus exit   |
| proof  | Option<EncodedSingleProof>  | the zk proof of exodus exit |
```rust
struct ProofInfo{
  id: ProofId,
  amount: Option<BigUintSerdeWrapper>,
  proof: Option<EncodedSingleProof>
}
```

## API
### Note
if recover state isn't completed, **tokens, running_max_task_id, get_token, get_stored_block_info, get_balances, 
get_unprocessed_priority_ops, get_proof_task_id, get_proof_by_info, get_proofs_by_id, generate_proof_task_by_info,
generate_proof_tasks_by_token** api return 
#### Response
```json
{
    "code": 70,
    "data": null,
    "err_msg": "Recovering state is unfinished"
}
```
### contracts
Get the ZkLink contract addresses of all chain.
#### GET Request
#### Response
```json
{
  "code": 0,
  "data": {
    "1": "0x517aa9dec0e297b744ac7ac8ddd8b127c1993055",
    "2": "0x331a96b91f35051706680d96251931e26f4ba58a"
  },
  "err_msg": null
}
```
Success returns a `HashMap<ChainId, ZkLinkAddress>`, Failure returns error description

### tokens
Get the info of all token.
#### GET Request
#### Response
if recover state isn't completed
```json
{
    "code": 40,
    "data": null,
    "err_msg": "Recovering state is unfinished"
}
```
```json
{
  "code": 0,
  "data": {
    "42": {
      "token_id": 42,
      "symbol": "wMATIC",
      "addresses": {
        "1": "0x76c9ef75f019496376c04dd19c38637cacce9e42"
      }
    },
    "1": {
      "token_id": 1,
      "symbol": "USD",
      "addresses": {}
    }
  }
}
```
Success returns a `HashMap<TokenId, TokenInfo>`, Failure returns error description

### recover_progress
Get the progress of the recover state, including the current synced block height and the total number of verified blocks.

#### GET Request
#### Response
```json
{
  "code": 0,
  "data": {
    "current_block": 10,
    "total_verified_block": 20
  },
  "err_msg": null
}
```
On success, it returns the current recover progress, including current_block (the current synced block height) and total_verified_block (the total number of verified blocks).

On failure, it returns an error description.

### running_max_task_id
Request to get max running task id.
#### GET Request
#### Response
```json
{
  "code": 0,
  "data": {
    "id": 0
  },
  "err_msg": null
}
```
On success, it returns the max id of task running. On failure, it returns an error description.

### get_token
Get token info(supported chains, token's contract addresses) by token_id
#### POST Request
```json
{
  "token_id": 17
}
```
#### Response
```json
{
  "code": 0,
  "data": {
    "token_id": 50,
    "symbol": "ETH3L",
    "addresses": {
      "2": "0x22343f93f70af0c88b25223111bcd35b9c8400dd"
    }
  },
  "err_msg": null
}
```
Success returns [`TokenInfo`](#TokenInfo), Failure returns error description

### get_stored_block_info
Get the stored last block info of the specified chain.
#### POST Request
```json
{
  "chain_id": 1
}
```
#### Response
```json
{
  "code": 0,
  "data": {
    "block_number": 60,
    "priority_operations": 0,
    "pending_onchain_operations_hash": "0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470",
    "timestamp": "0x64196341",
    "state_hash": "0x17dc6e99dbe4b15c76d8eca67d98c3197b08840cb736f811f55b15e032573967",
    "commitment": "0xe1f0e8e2849c138470d652f67caa26d830691ae942db15557740e56abfe5602d",
    "sync_hash": "0xbf45ccbdadba9dcf1eb155f3c36c51dec47bfa2c58af9b522c52d0adfee63ae0"
  },
  "err_msg": null
}
```
Success returns [`StoredBlockInfo`](#StoredBlockInfo), Failure returns error description

### get_balances
Get balances fo all token by ZkLinkAddress
#### POST Request
```json
{
  "address": "0x1aef2b4c06b83cdb2783d3458cdbf3886a6ae7d4"
}
```
#### Response
```json
{
  "code": 0,
  "data": {
    "0": {
      "18": "1498994167999999999973"
    },
    "1": {
      "31": "1498994167999999999973"
    }
  },
  "err_msg": null
}
```
Success returns `HashMap<SubAccountId,<TokenId,Balance>>`,
Failure returns error description

### get_unprocessed_priority_ops
Get all unprocessed priority ops by chain id
#### GET Request
```json
{
  "chain_id": 1
}
```
#### Response
```json
{
  "code": 0,
  "data": [
    {
      "serial_id": 80,
      "pub_data": {
        "Deposit": {
          "chain_id": 2,
          "sub_account_id": 1,
          "l2_target_token_id": 18,
          "l1_source_token_id": 18,
          "amount": "10000000000000000000",
          "owner": "0x3d809e414ba4893709c85f242ba3617481bc4126"
        }
      }
    },
    {
      "serial_id": 81,
      "pub_data": {
        "Deposit": {
          "chain_id": 2,
          "sub_account_id": 31,
          "l2_target_token_id": 18,
          "l1_source_token_id": 18,
          "amount": "70000000000000000000",
          "owner": "0x3d809e414ba4893709c85f242ba3617481bc4126"
        }
      }
    },
    {
      "serial_id": 82,
      "pub_data": {
        "Deposit": {
          "chain_id": 2,
          "sub_account_id": 0,
          "l2_target_token_id": 44,
          "l1_source_token_id": 44,
          "amount": "1500000000000000000000",
          "owner": "0x3d809e414ba4893709c85f242ba3617481bc4126"
        }
      }
    },
    {
      "serial_id": 83,
      "pub_data": {
        "Deposit": {
          "chain_id": 2,
          "sub_account_id": 0,
          "l2_target_token_id": 45,
          "l1_source_token_id": 45,
          "amount": "1350000000000000000000",
          "owner": "0x3d809e414ba4893709c85f242ba3617481bc4126"
        }
      }
    }
  ],
  "err_msg": null
}
```
Success returns `Vec<UnprocessedPriorityOp>`,
Failure returns error description

### get_proof_by_info
Get the proof by the specified [ExitInfo](#ExitProofData).
#### POST Request
```json
{
    "chain_id": 1,
    "account_address": "0x1aef2b4c06b83cdb2783d3458cdbf3886a6ae7d4", 
    "account_id": 12,
    "sub_account_id": 1,
    "l1_target_token": 17,
    "l2_source_token": 1
}
```
#### Response
correct 
```json
{
  "code": 0,
  "data": {
    "exit_info": {
      "chain_id": 1,
      "account_address": "0x1aef2b4c06b83cdb2783d3458cdbf3886a6ae7d4",
      "account_id": 12,
      "sub_account_id": 1,
      "l1_target_token": 17,
      "l2_source_token": 1
    },
    "proof_info": {
      "id": 1,
      "amount": "123456",
      "proof": "0x4566521312321321321321"
    }
  },
  "err_msg": null
}
```
```json
{
  "exit_info": {
    "chain_id": 1,
    "account_address": "0x0000000000000000000000000000000000000000000000000000000000000000",
    "account_id": 12,
    "sub_account_id": 1,
    "l1_target_token": 17,
    "l2_source_token": 1
  },
  "proof_info": {
    "id": 1,
    "amount": null,
    "proof": null
  }
}
```
wrong 
```json
{
  "code": 104,
  "data": null,
  "err_msg": "The exit proof task not exist"
}
```
Success returns [ExitProofData](#ExitProofData), Failure returns error description

### get_proof_task_id
Request to get the task id(proof id) by exit info
#### POST Request
```json
{
    "chain_id": 1,
    "account_address": "0x1aef2b4c06b83cdb2783d3458cdbf3886a6ae7d4", 
    "account_id": 12,
    "sub_account_id": 1,
    "l1_target_token": 17,
    "l2_source_token": 1
}
```
#### Response
```json
{
  "code": 0,
  "data": {
    "id": 1
  },
  "err_msg": null
}
```
Success returns the id, Failure returns error description

### get_proofs_by_id
Get the specified number of proofs closer to the id by passing the id
#### POST Request
```json
{
  "id": null,
  "proofs_num": 1
}
```
#### Response
```json
{
    "code": 0,
    "data": {
      "total_completed_num": 10,
      "proofs": [
        {
          "exit_info": {
            "chain_id": 2,
            "account_address": "0x0000000000000000000000000000000000000000000000000000000000000000",
            "account_id": 0,
            "sub_account_id": 0,
            "l1_target_token": 50,
            "l2_source_token": 50
          },
          "proof_info": {
            "id": 1,
            "amount": null,
            "proof": null
          }
        }
      ]
    },
    "err_msg": null
}
```
Success returns the vector of [ExitProofData](#ExitProofData), Failure returns error description

### get_proofs_by_token
Get all proofs of all blockchain by the specified ZkLinkAddress and TokenId and SubAccountId.
#### POST Request
```json
{
  "address": "0x04EBC47B5B0FA6E283DDC3C3B21DC9CD6B036D38",
  "sub_account_id": 1,
  "token_id": 1
}
```
#### Response
If token_id=1=USD, layer1 should withdraw each stable coin.
##### correct 
```json
{
  "code": 0,
  "data": [
    {
      "exit_info": {
        "chain_id": 1,
        "account_address": "0x04EBC47B5B0FA6E283DDC3C3B21DC9CD6B036D38",
        "account_id": 12,
        "sub_account_id": 1,
        "l1_target_token": 17,
        "l2_source_token": 1
      },
      "proof_info": {
        "id": 1,
        "amount": "123456",
        "proof": "0x4566521312321321321321"
      }
    },{
      "exit_info": {
        "chain_id": 1,
        "account_address": "0x04EBC47B5B0FA6E283DDC3C3B21DC9CD6B036D38",
        "account_id": 12,
        "sub_account_id": 1,
        "l1_target_token": 18,
        "l2_source_token": 1
      },
      "proof_info": {
        "id": 1,
        "amount": null,
        "proof": null
      }
    },{
      "exit_info": {
        "chain_id": 2,
        "account_address": "0x04EBC47B5B0FA6E283DDC3C3B21DC9CD6B036D38",
        "account_id": 12,
        "sub_account_id": 1,
        "l1_target_token": 17,
        "l2_source_token": 1
      },
      "proof_info": {
        "id": 1,
        "amount": null,
        "proof": null
      }
    },{
      "exit_info": {
        "chain_id": 2,
        "account_address": "0x04EBC47B5B0FA6E283DDC3C3B21DC9CD6B036D38",
        "account_id": 12,
        "sub_account_id": 1,
        "l1_target_token": 18,
        "l2_source_token": 1
      },
      "proof_info": {
        "id": 1,
        "amount": null,
        "proof": null
      }
    }
  ],
  "err_msg": null
}
```
wrong
```json
{
  "code": 102,
  "data": [],
  "err_msg": "The account not exist"
}
```
Success returns an array of element [ExitProofData](#ExitProofData), Failure returns error description

### generate_proof_task_by_info
Request to generate proof by [ExitInfo](#ExitProofData)
#### POST Request
```json
{
    "chain_id": 1,
    "account_address": "0x1aef2b4c06b83cdb2783d3458cdbf3886a6ae7d4", 
    "account_id": 12,
    "sub_account_id": 1,
    "l1_target_token": 17,
    "l2_source_token": 1
}
```
#### Response
```json
{
    "code": 0,
    "data": null,
    "err_msg": null
}
```
Success returns code=0, Failure returns error description

### generate_proof_tasks_by_token
Request to generate proof by the specified ZkLinkAddress and TokenId and SubAccountId
#### POST Request
```json
{
    "address": "0x04EBC47B5B0FA6E283DDC3C3B21DC9CD6B036D38",
    "sub_account_id": 1,
    "token_id": 1
}
```
#### Response
```json
{
    "code": 0,
    "data": null,
    "err_msg": null
}
```
Success returns code=0, Failure returns error description
