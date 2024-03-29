import { Address, L2ChainId, SubAccountId, TokenId, Wei } from '../../types/global'
import { ConnectorNames } from '../../connectors'

export interface NetworkInfo {
  name: string
  chainId: number
  layerTwoChainId: number
  symbol: string
  decimals: number
  rpcUrl: string
  explorerUrl: string
  iconUrl: string
}
export interface Contracts {
  [x: number]: string
}

export interface TokenInfo {
  token_id: TokenId
  symbol?: string
  addresses: {
    [x: L2ChainId]: Address
  }
}
export interface Tokens {
  [x: TokenId]: TokenInfo
}

export interface Balance {
  [x: SubAccountId]: {
    [x: TokenId]: Wei
  }
}

export interface RecoverProgress {
  blocks: number
}

export interface ProofInfo {
  exit_info: {
    chain_id: number
    account_address: string
    account_id: number
    sub_account_id: number
    l1_target_token: number
    l2_source_token: number
  }
  proof_info: {
    id: number
    amount: Wei | null
    proof: {
      inputs: string[]
      proof: string[]
    } | null
  }
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

export interface ProofHistory {
  total_completed_num: number
  proofs: ProofInfo[]
}
export interface PendingBalance extends TokenInfo {
  balance: Wei
}

export interface ProofInfoWithToken {
  chainId: L2ChainId
  accountAddress: Address
  subAccountId: SubAccountId
  l2SourceToken: TokenId
  l1TargetToken: TokenId
  proof: ProofInfo
}
export interface HomeState {
  account: Address
  networks: NetworkInfo[]
  currentChain?: NetworkInfo
  contracts?: Contracts
  exodusMode: {
    [x: L2ChainId]: number
  }
  totalBlocksExecuted: {
    [x: L2ChainId]: number
  }
  recoverProgress?: RecoverProgress
  runningTaskId: number
  connectorName?: ConnectorNames
  tokens: Tokens
  balance: Balance
  storedBlockInfos: {
    [x: L2ChainId]: StoredBlockInfo
  }
  proofs: ProofInfoWithToken[]
  multicallContracts?: Address[]
  proofHistory?: ProofHistory
  pendingBalance: {
    [x: Address]: PendingBalance[] | undefined
  }
}
