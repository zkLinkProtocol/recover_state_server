# zkLink types. Essential types for the zkLink network

`zklink_types` is a crate containing essential zkLink network types, such as transactions, operations and blockchain
primitives.

zkLink operations are split into the following categories:

- **transactions**: operations of zkLink network existing purely in the L2. Currently includes `Transfer`, `Withdraw`,
  `ChangePubKey` and `ForcedExit`. All the transactions form an enum named `zkLinkTx`.
- **priority operations**: operations of zkLink network which are triggered by invoking the zkLink smart contract method
  in L1. These operations are discovered by the zkLink server and included into the block just like L2 transactions.
  Currently includes `Deposit` and `FullExit`. All the priority operations form an enum named `zkLinkPriorityOp`.
- **operations**: a superset of `zkLinkTx` and `zkLinkPriorityOp`. All the operations are included into an enum named
  `zkLinkOp`. This enum contains all the items that can be included into the block, together with meta-information about
  each transaction. Main difference of operation from transaction/priority operation is that it can form public data
  required for the committing the block on the L1.

## License

`zklink_types` is a part of zkLink stack, which is distributed under the terms of both the MIT license and the Apache
License (Version 2.0).

See [LICENSE-APACHE](../../LICENSE-APACHE), [LICENSE-MIT](../../LICENSE-MIT) for details.
