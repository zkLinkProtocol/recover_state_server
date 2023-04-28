
| configuration variables                        | description                                                  | example                                                              |
|------------------------------------------------|--------------------------------------------------------------|----------------------------------------------------------------------|
| `RUNTIME_CONFIG_ZKLINK_HOME`                   | The current project path                                     | /home/xxx_user/recover_state_server                                  |
| `DATABASE_URL`                                 | the default is local.                                        | postgres://user:passwd@localhost/plasma                              |
| `CHAIN_IDS`                                    | The chains that supported, the chain id is defined by zkLink | 1,2                                                                  |
| `CHAIN_{CHAIN_ID}_CHAIN_ID`                    | The chain ID defined by zkLink                               | 1                                                                    |
| `CHAIN_{CHAIN_ID}_CHAIN_TYPE`                  | The layer1 chain type                                        | EVM                                                                  |
| `CHAIN_{CHAIN_ID}_GAS_TOKEN`                   | The gas token price symbol                                   | MATIC                                                                |
| `CHAIN_{CHAIN_ID}_IS_COMMIT_COMPRESSED_BLOCKS` | Whether the data is fully on-chain in this chain             | true                                                                 |
| `CHAIN_{CHAIN_ID}_CONTRACT_DEPLOYMENT_BLOCK`   | The block number of CONTRACT deployed                        | 33377564                                                             |
| `CHAIN_{CHAIN_ID}_CONTRACT_ADDRESS`            | The zkLink main contract address                             | "0x517aa9dec0E297B744aC7Ac8ddd8B127c1993055"                         |
| `CHAIN_{CHAIN_ID}_CONTRACT_GENESIS_TX_HASH`    | The zkLink contract deployed tx hash                         | "0x5c576039ffefce307ffbc5556899ee0772efcf2046051cc4fe9ca633987061ca" |
| `CHAIN_{CHAIN_ID}_CLIENT_CHAIN_ID`             | The real chain id defined in layer1                          | 80001                                                                |
