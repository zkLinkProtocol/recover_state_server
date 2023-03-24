CREATE TYPE layer1_account_type AS ENUM (
    'Unknown',
    'EthOwned',
    'EthCREATE2',
    'StarkContract'
);

CREATE TYPE agg_type AS ENUM (
    'CommitBlocks',
    'CreateProofBlocks',
    'PublishProofBlocksOnchain',
    'BridgeBlocks',
    'SyncBlocks',
    'ExecuteBlocks'
);

-- token info and price
-- USD(0), USDX(1-16) also stored in this table
-- USD(0) will be stored in genesis block,
-- USDX(1-16) will be stored when stable token added
CREATE TABLE token_price (
    token_id integer NOT NULL, -- token id
    symbol text NOT NULL, -- token symbol
    price_id text NOT NULL, -- id defined by price service used for query token price
    usd_price numeric NOT NULL, -- token price
    last_update_time timestamp with time zone NOT NULL, -- token price last update time

    PRIMARY KEY (token_id)
);

-- token configs of each chain
-- only layer one token chain info stored in this table
CREATE TABLE tokens (
    id integer NOT NULL REFERENCES token_price(token_id) ON UPDATE CASCADE ON DELETE CASCADE, -- token id，the same token id on different chains must be the same
    chain_id smallint NOT NULL, -- chain id of zkLink
    address bytea NOT NULL, -- token address，the same token address on different chains will be different
    decimals smallint NOT NULL, -- token decimals of layer one
    fast_withdraw boolean NOT NULL, -- true means it support fast withdraw

   PRIMARY KEY (id, chain_id)
);

-- dynamic state of each layer one chain
-- this table will be inited according to ZkLink config in genesis block
CREATE TABLE eth_parameters (
    chain_id smallint NOT NULL, -- chain id of zkLink
    gas_token_id integer NOT NULL REFERENCES token_price(token_id) ON UPDATE CASCADE ON DELETE CASCADE, -- gas token id of layer one
    average_gas_price numeric NOT NULL, -- tx average gas price of layer one

    PRIMARY KEY (chain_id)
);

-- account state after blocked
-- we can not set account type before setting pubkey hash
-- 0 is fee account, address is the fee address
-- 1 is global asset account, address is 0xffffffffffffffffffffffffffffffffffffffff
-- 0 and 1 accounts both init when created the genesis block
CREATE TABLE accounts (
    id bigserial NOT NULL, -- account id
    nonce bigint NOT NULL, -- account nonce
    address bytea NOT NULL, -- account address
    pubkey_hash bytea NOT NULL, -- account pubkey hash， default is zero hash if account is not set
    account_type layer1_account_type NOT NULL, -- account type, eg. EthOwned, EthCREATE2, StarkContract
    chain_id smallint NOT NULL, -- when account type is EthCREATE2, the value is chain id of `ChangePubKey`
    last_block bigint NOT NULL, -- the last block of account update(nonce, address or pubkey_hash) transaction within

    PRIMARY KEY(id),
    UNIQUE (address)
);
CREATE INDEX accounts_block_index ON accounts USING btree (last_block);
CREATE INDEX accounts_pubkey_hash_index ON accounts USING btree (pubkey_hash);

-- token balances of (account, sub_account) after blocked
CREATE TABLE balances (
    account_id bigint NOT NULL REFERENCES accounts(id) ON UPDATE CASCADE ON DELETE CASCADE, -- account id
    sub_account_id integer NOT NULL, -- sub account id
    coin_id integer NOT NULL, -- token id
    balance numeric NOT NULL, -- token balance

    PRIMARY KEY (account_id, sub_account_id, coin_id)
);
CREATE INDEX balances_account_idx ON balances USING btree (account_id);
CREATE INDEX balances_sub_account_idx ON balances USING btree (sub_account_id);

-- slot order nonce of (account, sub_account) after blocked
CREATE TABLE account_order_nonces (
    account_id bigint NOT NULL REFERENCES accounts(id) ON UPDATE CASCADE ON DELETE CASCADE, -- account id
    sub_account_id integer NOT NULL, -- sub account id
    slot_id integer NOT NULL, -- slot id
    order_nonce bigint NOT NULL, -- order nonce of slot
    residue numeric NOT NULL, -- order amount residue after trade

    PRIMARY KEY (account_id, sub_account_id, slot_id)
);
CREATE INDEX account_order_nonces_account_idx ON account_order_nonces USING btree (account_id);
CREATE INDEX account_order_nonces_sub_account_idx ON account_order_nonces USING btree (sub_account_id);

-- account create record when block is accepted by layer 2
CREATE TABLE account_creates (
    account_id bigserial NOT NULL, -- new account id
    address bytea NOT NULL, -- account address
    block_number bigint NOT NULL, -- block which contains this record
    update_order_id integer NOT NULL, -- record index in block
    tx_hash bytea NOT NULL, -- tx which contains this record

    PRIMARY KEY (account_id)
);
CREATE INDEX account_creates_address_idx ON account_creates USING btree (address);
CREATE INDEX account_creates_block_index ON account_creates USING btree (block_number);
CREATE INDEX account_creates_tx_hash_index ON account_creates USING btree (tx_hash);

-- account update record  when block is accepted by layer 2
CREATE TABLE account_balance_updates (
    balance_update_id bigserial NOT NULL , -- record id
    account_id bigint NOT NULL, -- account id
    sub_account_id integer NOT NULL, -- sub account id
    coin_id integer NOT NULL, -- token id
    old_balance numeric NOT NULL, -- balance before update
    new_balance numeric NOT NULL, -- balance after update
    old_nonce bigint NOT NULL, -- account nonce before update
    new_nonce bigint NOT NULL, -- account nonce after update
    block_number bigint NOT NULL, -- block which contains this record
    update_order_id integer NOT NULL, -- record index in block
    tx_hash bytea NOT NULL, -- tx which contains this record

    PRIMARY KEY (balance_update_id)
);
CREATE INDEX account_balance_updates_block_index ON account_balance_updates USING btree (block_number);
CREATE INDEX account_balance_updates_account_index ON account_balance_updates USING btree (account_id);
CREATE INDEX account_balance_updates_sub_account_index ON account_balance_updates USING btree (sub_account_id);
CREATE INDEX account_balance_updates_coin_index ON account_balance_updates USING btree (coin_id);
CREATE INDEX account_balance_updates_tx_hash_index ON account_balance_updates USING btree (tx_hash);

-- order nonce update record when block is accepted by layer 2
-- order nonce update will not change account nonce
-- old_order_nonce and new_order_nonce are json arrays [order_nonce, residue], for example:
-- [14539,0]
CREATE TABLE account_order_updates (
  order_nonce_update_id bigserial NOT NULL, -- record id
  account_id bigint NOT NULL, -- account id
  sub_account_id integer NOT NULL, -- sub account id
  slot_id integer NOT NULL, -- slot id
  old_order_nonce jsonb NOT NULL, -- slot order nonce info before update
  new_order_nonce jsonb NOT NULL, -- slot order nonce info after update
  block_number bigint NOT NULL, -- block which contains this record
  update_order_id integer NOT NULL, -- record index in block
  tx_hash bytea NOT NULL, -- tx which contains this record

  PRIMARY KEY (order_nonce_update_id)
);
CREATE INDEX account_order_updates_block_index ON account_order_updates USING btree (block_number);
CREATE INDEX account_order_updates_account_index ON account_order_updates USING btree (account_id);
CREATE INDEX account_order_updates_sub_account_index ON account_order_updates USING btree (sub_account_id);
CREATE INDEX account_order_updates_slot_index ON account_order_updates USING btree (slot_id);
CREATE INDEX account_order_updates_tx_hash_index ON account_order_updates USING btree (tx_hash);

-- pubkey update record when block is accepted by layer 2
CREATE TABLE account_pubkey_updates (
   pubkey_update_id serial NOT NULL, -- record id
   account_id bigint NOT NULL, -- account id
   old_pubkey_hash bytea NOT NULL, -- account pubkey before update
   new_pubkey_hash bytea NOT NULL, -- account pubkey after update
   old_nonce bigint NOT NULL, -- account nonce before update
   new_nonce bigint NOT NULL, -- account nonce after update
   block_number bigint NOT NULL, -- block which contains this record
   update_order_id integer NOT NULL, -- record index in block
   tx_hash bytea NOT NULL, -- tx which contains this record

   PRIMARY KEY (pubkey_update_id)
);
CREATE INDEX account_pubkey_updates_block_index ON account_pubkey_updates USING btree (block_number);
CREATE INDEX account_pubkey_updates_account_index ON account_pubkey_updates USING btree (account_id);
CREATE INDEX account_pubkey_updates_tx_hash_index ON account_pubkey_updates USING btree (tx_hash);

-- tx table, priority operations from watcher and non-priority operations from rpc will be recorded to this table
CREATE TABLE submit_txs (
    id bigserial NOT NULL, -- tx id
    chain_id smallint NOT NULL, -- chain id of Deposit, FullExit, zero for other ops
    op_type smallint NOT NULL, -- operation type, eg. Deposit, Transfer
    from_account bytea NOT NULL, -- Deposit: sender, Transfer: from, OrderMatch:submitter
    to_account bytea NOT NULL, -- Deposit: receiver, Transfer: to
    nonce bigint NOT NULL, -- Deposit and FullExit: serial_id of event, other ops is the nonce of from_account
    amount numeric NOT NULL, -- the amount of Deposit, Transfer, Withdraw
    tx_data jsonb NOT NULL, -- zklink tx serialize data
    eth_signature jsonb, -- signature of layer one
    tx_hash bytea NOT NULL, -- tx hash，hash data contains op_type, tx hash of different op will not be conflicted
    created_at timestamp with time zone NOT NULL , -- create time
    executed boolean NOT NULL, -- whether the tx is trying to be included in a block
    executed_timestamp timestamp with time zone, -- execute time
    success boolean NOT NULL, -- if execute success
    fail_reason text, -- execute fail reason
    block_number bigint NOT NULL, -- if execute success, tx will be included in a block
    block_index integer NOT NULL, -- tx index in block
    operation jsonb, -- the operation data of executed in layer 1，eg. FullExit will contains amount to withdraw

    PRIMARY KEY (id),
    UNIQUE (tx_hash)
);
CREATE INDEX submit_txs_chain_id_index ON submit_txs USING btree (chain_id);
CREATE INDEX submit_txs_op_type_index ON submit_txs USING btree (op_type);
CREATE INDEX submit_txs_from_account_index ON submit_txs USING btree (from_account);
CREATE INDEX submit_txs_to_account_index ON submit_txs USING btree (to_account);
CREATE INDEX submit_txs_nonce_index ON submit_txs USING btree (nonce);
CREATE INDEX submit_txs_executed_index ON submit_txs USING btree (executed);
CREATE INDEX submit_txs_success_index ON submit_txs USING btree (success);
CREATE INDEX submit_txs_block_number_index ON submit_txs USING btree (block_number);
CREATE INDEX submit_txs_block_index ON submit_txs USING btree (block_index);

-- layer 2 blocks，only successful tx will be include in block
CREATE TABLE blocks (
    number bigserial NOT NULL, -- block number
    root_hash bytea NOT NULL, -- the root hash of state tree
    fee_account_id bigint NOT NULL, -- fee account id
    block_size bigint NOT NULL, -- the trunk number of block，when the proof is generated, the corresponding vk will be found according to the number of trunks
    ops_composition_number bigint NOT NULL, -- another selector to find vk，trunk number first，and then ops_composition_number
    created_at timestamp with time zone NOT NULL , -- block create time
    commitment bytea NOT NULL, -- block commitment
    sync_hash bytea NOT NULL, -- block sync hash
    commit_gas_limit bigint NOT NULL, -- the gas used upper limit in all chains of commit block to layer 1 in a batch
    verify_gas_limit bigint NOT NULL, -- the gas used upper limit in all chains of verify block to layer 1 in a batch

    PRIMARY KEY(number)
);
CREATE INDEX blocks_root_hash_index ON blocks USING btree (root_hash);

-- the proof of block，we can delete proof info after block verified in layer 1
-- the creation of block, witness and proof are async progress
-- when witness created it will be stored in this table
-- when proof received from prove client, it will be updated in this table
CREATE TABLE proofs (
   block_number bigint NOT NULL, -- block number of layer 2
   status smallint NOT NULL DEFAULT 1, -- 1: witness created, 2: proof created
   witness text NOT NULL, -- block witness
   proof jsonb, -- block proof
   created_at timestamp with time zone, -- witness create time

   PRIMARY KEY (block_number)
);
CREATE INDEX proofs_status ON proofs USING btree (status);

-- block and aggregate proof job
-- block proof only contains one block，aggregate proof may also contains only one block
CREATE TABLE prover_job_queue (
    id bigserial NOT NULL, -- job id
    job_status integer NOT NULL, -- job status, 0: idle, 1: in progress, 2: done
    job_priority integer NOT NULL, -- job priority
    job_type text NOT NULL, -- job type，block proof or aggregate proof
    created_at timestamp with time zone NOT NULL, -- job create time
    first_block bigint NOT NULL, -- start block of proof
    last_block bigint NOT NULL, -- end block of proof
    job_data jsonb NOT NULL, -- job data
    updated_by text, -- update prover id
    updated_at timestamp with time zone, -- job update time

    PRIMARY KEY (id)
);
CREATE INDEX prover_job_queue_job_status ON prover_job_queue USING btree (job_status);
CREATE INDEX prover_job_queue_job_job_priority ON prover_job_queue USING btree (job_priority);
CREATE INDEX prover_job_queue_job_type ON prover_job_queue USING btree (job_type);
CREATE INDEX prover_job_queue_first_block ON prover_job_queue USING btree (first_block);
CREATE INDEX prover_job_queue_last_block ON prover_job_queue USING btree (last_block);
CREATE INDEX prover_job_queue_updated_at ON prover_job_queue USING btree (updated_at);

-- aggregate operation
CREATE TABLE aggregate_operations (
    id bigserial NOT NULL, -- agg op id
    action_type agg_type NOT NULL, -- agg type: CreateProof, Commit, PublishProof, Bridge, Sync and Execute
    from_block bigint NOT NULL, -- from block number of agg op
    to_block bigint NOT NULL, -- to block number of agg op
    created_at timestamp with time zone NOT NULL, -- create time
    confirmed boolean NOT NULL, -- CreateProof need to wait prove committed by prove client, other agg types need to wait txs confirmed by layer 1

    PRIMARY KEY (id)
);
CREATE INDEX aggregate_ops_action_type ON aggregate_operations USING btree (action_type);
CREATE INDEX aggregate_operations_from_block ON aggregate_operations USING btree (from_block);
CREATE INDEX aggregate_operations_to_block ON aggregate_operations USING btree (to_block);
CREATE INDEX aggregate_ops_confirmed ON aggregate_operations USING btree (confirmed);

-- proof for CreateProof, CreateProof can be set to confirmed when proof is committed
-- and we can not create new PublishProof if proof is not exist
CREATE TABLE aggregated_proofs (
    id bigint NOT NULL REFERENCES aggregate_operations(id) ON UPDATE CASCADE, -- aggregate id of CreateProof
    proof jsonb NOT NULL, -- proof need for PublishProof
    first_block bigint NOT NULL, --  The first block number of the batch block of the proof
    last_block bigint NOT NULL, -- The last block number of the batch block of the proof
    created_at timestamp with time zone NOT NULL, -- proof create time

    PRIMARY KEY (id)
);
CREATE INDEX aggregated_proofs_first_block ON aggregated_proofs USING btree (first_block);
CREATE INDEX aggregated_proofs_last_block ON aggregated_proofs USING btree (last_block);

-- tx send to layer 1
-- Commit, PublishProof, Bridge, Sync and Execute will create some txs to send
-- only all txs confirmed by layer 1 and then these agg ops can set confirmed to true
CREATE TABLE eth_operations (
    id bigserial NOT NULL, -- layer 1 operation
    chain_id smallint NOT NULL, -- zkLink chain id
    nonce bigint, -- tx nonce of layer 1
    sent boolean NOT NULL, -- if tx sent to layer 1
    confirmed boolean NOT NULL, -- if tx confirmed by layer 1
    op_type agg_type NOT NULL, -- agg op type
    last_deadline_block bigint NOT NULL, -- the dead line block that tx confirmed by layer 1，tx will be considered stuck if not confirmed over this block number
    last_used_gas_price numeric NOT NULL, -- gas price of last tx send to layer 1, if last tx is stuck, new tx will use a higher gas price to resend
    raw_tx jsonb NOT NULL, -- raw tx data
    gas_limit integer NOT NULL, -- tx send gas limit
    final_hash bytea, -- confirmed tx hash of layer 1, tx may be resubmitted to layer 1 if tx is stuck

    PRIMARY KEY (id)
);
CREATE INDEX eth_operations_chain_id_idx ON eth_operations USING btree (chain_id);
CREATE INDEX eth_operations_confirmed ON eth_operations USING btree (confirmed);
CREATE INDEX eth_operations_sent ON eth_operations USING btree (sent);

CREATE TABLE eth_aggregated_ops_binding (
    id bigserial NOT NULL,
    op_id bigint NOT NULL REFERENCES aggregate_operations(id),
    eth_op_id bigint NOT NULL REFERENCES eth_operations(id),

    PRIMARY KEY (id)
);
CREATE INDEX eth_agg_op_binding_idx ON eth_aggregated_ops_binding USING btree (op_id);

-- one eth operation may produce multiple layer 1 tx
CREATE TABLE eth_tx_hashes (
    id bigserial NOT NULL,
    chain_id smallint NOT NULL, -- zkLink chain id
    eth_op_id bigint NOT NULL REFERENCES eth_operations(id), -- id of eth_operations
    tx_hash bytea NOT NULL, -- tx hash of layer 1
    gas_price numeric NOT NULL, -- gas price of this transaction
    gas_used bigint, -- gas used of this transaction

    PRIMARY KEY (id)
);
CREATE INDEX eth_tx_hashes_eth_op_id_index ON eth_tx_hashes USING btree (eth_op_id);

-- store the parse progress of layer event
CREATE TABLE priority_op_progress (
    chain_id smallint NOT NULL, -- the source chain id(layer2) of priority op
    last_priority_block bigint NOT NULL, -- Which block does the watcher process the priority transaction from chain_id chain to
    last_serial_id bigint NOT NULL, -- Which serial id the watcher process the priority transaction from chain_id chain to

    PRIMARY KEY (chain_id)
);

-- address that can submit certain types of tx
CREATE TABLE tx_submitter_whitelist (
    id bigserial NOT NULL,
    sub_account_id integer NOT NULL, -- sub account id
    submitter_account_id bigint REFERENCES accounts(id) ON UPDATE CASCADE ON DELETE CASCADE NOT NULL, -- submitter in whitelist

    PRIMARY KEY (id)
);
CREATE INDEX tx_submitter_whitelist_sub_account_id_index ON tx_submitter_whitelist USING btree (sub_account_id);
CREATE INDEX tx_submitter_whitelist_submitter_account_id_index ON tx_submitter_whitelist USING btree (submitter_account_id);


-- -------------------- --
-- Data restore section --
-- -------------------- --
CREATE TABLE recover_state_events_state (
    id serial PRIMARY KEY,
    block_type text NOT NULL,
    transaction_hash bytea NOT NULL,
    block_num bigint NOT NULL,
    contract_version smallint NOT NULL
);

CREATE TABLE recover_state_storage_state_update
(
    id serial PRIMARY KEY,
    storage_state text NOT NULL
);

CREATE TABLE recover_state_last_watched_block
(
    chain_id smallint NOT NULL,
    event_type text NOT NULL,
    block_number bigint NOT NULL,
    last_serial_id bigint NOT NULL, -- Which serial id the watcher process the priority transaction from chain_id chain to

    PRIMARY KEY (chain_id, event_type)
);

CREATE TABLE recover_state_rollup_ops
(
    block_num bigserial NOT NULL,
    operation jsonb  NOT NULL,
    fee_account bigint NOT NULL,
    created_at timestamp with time zone,
    previous_block_root_hash bytea NOT NULL,
    contract_version smallint not null,

    PRIMARY KEY (block_num)
);

CREATE TABLE exit_proofs
(
    chain_id smallint NOT NULL,
    account_id bigserial NOT NULL,
    sub_account_id smallint NOT NULL,
    l1_target_token integer NOT NULL,
    l2_source_token integer NOT NULL,
    proof jsonb, -- the proof of exodus exit
    amount numeric, -- the amount of exodus exit in chain_id
    created_at timestamp with time zone,
    finished_at timestamp with time zone,

    PRIMARY KEY (chain_id, account_id, sub_account_id, l1_target_token, l2_source_token)
);

