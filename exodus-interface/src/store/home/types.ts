import { Address, L2ChainId, SubAccountId, TokenId, Wei } from '../../types/global'
import { ChainInfo } from '../../config/chains'
import { ConnectorNames } from '../../connectors'

export interface Contracts {
  [x: number]: string
}

export interface Tokens {
  [x: TokenId]: {
    token_id: TokenId
    symbol?: string
    addresses: {
      [x: L2ChainId]: Address
    }
  }
}

export interface Balances {
  [x: SubAccountId]: {
    [x: TokenId]: Wei
  }
}

// export interface BalanceInfo {
//   subAccountId: SubAccountId
//   tokenId: TokenId
//   balance: Wei
// }

export interface ProofInfo {
  exit_info: {
    chain_id: number
    account_address: string
    account_id: number
    sub_account_id: number
    l1_target_token: number
    l2_source_token: number
  }
  amount: Wei | null
  proof: {
    inputs: string[]
    proof: string[]
  } | null
}

export interface StoredBlockInfo {
  block_number: number
  priority_operations: number
  pending_onchain_operations_hash: string
  timestamp: string
  state_hash: string
  commitment: string
  sync_hash: string
}

export interface HomeState {
  currentChain: ChainInfo | undefined
  contracts: Contracts | undefined
  connectorName: ConnectorNames | undefined
  tokens: Tokens
  balances: Balances
  storedBlockInfos: {
    [x: L2ChainId]: StoredBlockInfo
  }
  proofs: {
    [a: SubAccountId]: {
      [x: TokenId]: ProofInfo[]
    }
  }
  multicallContracts: Address[] | undefined
}
