# ZkLink Exodus Prover

A prover that generates zero-knowledge proofs for the ZkLink exodus model.

## Usage
```
zklink-exodus-prover [OPTIONS] <SUBCOMMAND>
```
### Options

- `--help`: Prints help information.
- `--version`: Prints version information.

### Subcommands

#### `tasks`

Runs the prover tasks module for running programmers.

##### Options

- `-w, --workers_num [WORKERS_NUM]`: The number of workers required to run.

##### Example
```
zklink-exodus-prover tasks --workers_num 5
```

#### `single`

Generates a single proof based on the specified exit information for a chain to withdraw.

##### Options

- `-c, --chain_id [CHAIN_ID]`: Chain to withdraw (default: `1`).
- `-i, --account_id [ACCOUNT_ID]`: Account ID of the account (cannot be negative or 1) (default: `0`).
- `-s, --sub-account-id [SUB_ACCOUNT_ID]`: Sub-account ID of the account (default: `0`).
- `--l1_target_token [L1_TARGET_TOKEN]`: Target token to withdraw to layer1 (default: `USDT`).
- `--l2_source_token [L2_SOURCE_TOKEN]`: Source token to withdraw from layer2 (default: `USD`).

##### Example
```
zklink-exodus-prover single -c 1 -i 0 -s 0 --l1_target_token USDT --l2_source_token USD
```

## Authors

- N Labs

## License

This project is licensed under the MIT License - see the `LICENSE` file for details.
