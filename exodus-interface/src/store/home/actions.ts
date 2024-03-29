import { createAction, createAsyncThunk } from '@reduxjs/toolkit'
import {
  Tokens,
  HomeState,
  Balance,
  StoredBlockInfo,
  ProofInfo,
  RecoverProgress,
  ProofHistory,
  NetworkInfo,
  PendingBalance,
  ProofInfoWithToken,
} from './types'
import { Address, ChainId, L2ChainId, SubAccountId, TokenId, Wei } from '../../types/global'
import { DunkirkResponse, http } from '../../api'
import axios, { AxiosResponse } from 'axios'
import { STATIC_HOST } from '../../config'
import { Web3Provider } from '@ethersproject/providers'
import { RootState } from '..'
import { Interface } from 'ethers/lib/utils'
import MainContract from 'zklink-js-sdk/abi/ZkLink.json'
import { BigNumber } from 'ethers'

export const updateCurrentAccount = createAction<Address>('home/updateCurrentAccount')
export const updateCurrentChain = createAction<NetworkInfo | undefined>('home/updateCurrentChain')
export const updateConnectorName = createAction<string>('home/updateConnectorName')
export const updateContracts = createAction<HomeState['contracts']>('home/updateContracts')
export const updateTokens = createAction<Tokens>('home/updateTokens')
export const updateBalances = createAction<Balance>('home/updateBalances')
export const updateStoredBlockInfo = createAction<{
  chainId: L2ChainId
  storedBlockInfo: StoredBlockInfo
}>('home/updateStoredBlockInfo')
export const updateProofs = createAction<{
  address: Address
  sub_account_id: number
  token_id: number
}>('home/updateProofs')

export const fetchNetworks = createAsyncThunk<NetworkInfo[]>('home/fetchNetworks', async () => {
  const r: AxiosResponse<NetworkInfo[]> = await axios.get(`${STATIC_HOST}/networks/list.json`)
  return r.data
})
interface ProofsArgs {
  chain_id: L2ChainId
  account_address: Address
  sub_account_id: number
  l2_source_token: TokenId
  l1_target_token: TokenId
}
export const fetchProofs = createAsyncThunk<ProofInfoWithToken, ProofsArgs>(
  'home/fetchProofs',
  async (args) => {
    const r = await http.post('/get_proof_by_info', {
      ...args,
      account_id: 0,
    })

    return {
      chainId: args.chain_id,
      accountAddress: args.account_address,
      subAccountId: args.sub_account_id,
      l2SourceToken: args.l2_source_token,
      l1TargetToken: args.l1_target_token,
      proof: r.data.data,
    }
  }
)
export const fetchRecoverProgress = createAsyncThunk<RecoverProgress>(
  'home/fetchRecoverProgress',
  async () => {
    const r: AxiosResponse<DunkirkResponse<RecoverProgress>> = await axios.get('/server/blocks')
    return r.data.data
  }
)
export const fetchRunningTaskId = createAsyncThunk<number>('home/fetchRunningTaskId', async () => {
  const r: AxiosResponse<
    DunkirkResponse<{
      id: number
    }>
  > = await http.get('/running_max_task_id')
  return r.data.data.id
})
export const fetchProofHistory = createAsyncThunk<
  ProofHistory,
  {
    page?: number
    proofs_num?: number
  }
>('home/fetchProofHistory', async ({ page = 0, proofs_num = 10 }) => {
  const r: AxiosResponse<DunkirkResponse<ProofHistory>> = await http.post('/get_proofs_by_page', {
    page,
    proofs_num,
  })
  return r.data.data
})
export const fetchPendingBalances = createAsyncThunk<
  {
    account: Address
    balances: PendingBalance[]
  },
  {
    provider: Web3Provider
    account: Address
  },
  {
    state: RootState
  }
>('home/fetchPendingBalances', async ({ provider, account }, { getState }) => {
  const state = getState()
  const { tokens, contracts, currentChain } = state.home

  if (!tokens || !contracts || !currentChain) {
    return Promise.reject()
  }

  const balances = []
  for (let i in tokens) {
    const iface = new Interface(MainContract.abi)
    const fragment = iface.getFunction('getPendingBalance')
    const calldata = iface.encodeFunctionData(fragment, [account, i])

    const r = await provider.send('eth_call', [
      {
        from: account,
        to: contracts[currentChain.layerTwoChainId],
        data: calldata,
      },
    ])

    if (BigNumber.from(r).isZero()) {
      continue
    }

    const b = { ...tokens[i], balance: r }
    balances.push(b)
  }
  return {
    account,
    balances,
  }
})
export const updatePendingBalances = createAction<{
  account: Address
  balances: PendingBalance[] | undefined
}>('home/updatePendingBalances')

export const fetchExodusMode = createAsyncThunk<
  {
    chainId: L2ChainId
    exodusMode: number
  },
  {
    provider: Web3Provider
  },
  {
    state: RootState
  }
>('home/fetchExodusMode', async ({ provider }, { getState }) => {
  const state = getState()
  const { contracts, currentChain } = state.home

  if (!contracts || !currentChain) {
    return Promise.reject()
  }

  const iface = new Interface(MainContract.abi)
  const fragment = iface.getFunction('exodusMode')
  const calldata = iface.encodeFunctionData(fragment, [])

  const r = await provider.send('eth_call', [
    {
      to: contracts[currentChain.layerTwoChainId],
      data: calldata,
    },
  ])

  return {
    chainId: currentChain.layerTwoChainId,
    exodusMode: Number(r), // 0 | 1
  }
})

export const fetchTotalBlocksExecuted = createAsyncThunk<
  {
    chainId: L2ChainId
    totalBlocksExecuted: number
  },
  {
    provider: Web3Provider
  },
  {
    state: RootState
  }
>('home/fetchTotalBlocksExecuted', async ({ provider }, { getState }) => {
  const state = getState()
  const { contracts, currentChain } = state.home

  if (!contracts || !currentChain) {
    return Promise.reject()
  }

  const iface = new Interface(MainContract.abi)
  const fragment = iface.getFunction('totalBlocksExecuted')
  const calldata = iface.encodeFunctionData(fragment, [])

  const r = await provider.send('eth_call', [
    {
      to: contracts[currentChain.layerTwoChainId],
      data: calldata,
    },
  ])

  return {
    chainId: currentChain.layerTwoChainId,
    totalBlocksExecuted: Number(r),
  }
})
