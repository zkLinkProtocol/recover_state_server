# ZkLink exodus prover
A command-line tool for running prover tasks and generating a single proof based on the specified exit information.

## build
Once you have Rust and Cargo installed, you can install this tool by running the following command:
```
cargo build --release
```

### Usage
To run the prover tasks module, use the following command:
```
./target/release/zklink_exodus_prover tasks
```
To generate a single proof based on the specified exit information, use the following command:

```
./target/release/zklink_exodus_prover single --chain_id <CHAIN_ID> --account_id <ACCOUNT_ID> --sub_account_id <SUB_ACCOUNT_ID> --l1_target_token <L1_TARGET_TOKEN> --l2_source_token <L2_SOURCE_TOKEN>
```
Replace <CHAIN_ID>, <ACCOUNT_ID>, <SUB_ACCOUNT_ID>, <L1_TARGET_TOKEN>, and <L2_SOURCE_TOKEN> with the appropriate values.

### Here are the descriptions of the command-line arguments:
- chain_id: The chain to withdraw from. This is a required argument.
- account_id: The account ID of the account to withdraw from. This is a required argument.
- sub_account_id: The sub_account ID of the account to withdraw from. This is a required argument.
- l1_target_token: The target token to withdraw to layer 1. This is a required argument.
- l2_source_token: The source token to withdraw from layer 2. This is a required argument.