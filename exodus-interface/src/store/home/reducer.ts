import { createReducer } from '@reduxjs/toolkit'
import {
  fetchNetworks,
  fetchPendingBalances,
  fetchProofHistory,
  fetchProofs,
  fetchRecoverProgress,
  fetchRunningTaskId,
  updateBalances,
  updateContracts,
  updateCurrentAccount,
  updateCurrentChain,
  updatePendingBalances,
  updateStoredBlockInfo,
  updateTokens,
} from './actions'
import { HomeState } from './types'

const initialState: HomeState = {
  account: '',
  networks: [],
  currentChain: undefined,
  contracts: {},
  recoverProgress: undefined,
  runningTaskId: 0,
  connectorName: undefined,
  tokens: {},
  balances: {},
  storedBlockInfos: {},
  proofs: {},
  multicallContracts: undefined,
  proofHistory: undefined,
  pendingBalances: {},
}

export default createReducer<HomeState>(initialState, (builder) => {
  builder
    .addCase(updateCurrentAccount, (state, { payload }) => {
      state.account = payload
    })
    .addCase(fetchNetworks.fulfilled, (state, { payload }) => {
      state.networks = payload
    })
    .addCase(updateCurrentChain, (state, { payload }) => {
      state.currentChain = payload
    })
    .addCase(updateContracts, (state, { payload }) => {
      state.contracts = payload
    })
    .addCase(updateTokens, (state, { payload }) => {
      state.tokens = payload
    })
    .addCase(updateBalances, (state, { payload }) => {
      state.balances = payload
    })
    .addCase(updateStoredBlockInfo, (state, { payload }) => {
      state.storedBlockInfos[payload.chainId] = payload.storedBlockInfo
    })
    .addCase(fetchProofs.fulfilled, (state, { payload }) => {
      if (!state.proofs[payload.subAccountId]) {
        state.proofs[payload.subAccountId] = {}
      }
      state.proofs[payload.subAccountId][payload.tokenId] = payload.data
    })
    .addCase(fetchRecoverProgress.fulfilled, (state, { payload }) => {
      state.recoverProgress = payload
    })
    .addCase(fetchRunningTaskId.fulfilled, (state, { payload }) => {
      state.runningTaskId = payload
    })
    .addCase(fetchProofHistory.fulfilled, (state, { payload }) => {
      state.proofHistory = payload
    })
    .addCase(fetchPendingBalances.fulfilled, (state, { payload }) => {
      state.pendingBalances[payload.account] = payload.balances
    })
    .addCase(updatePendingBalances, (state, { payload }) => {
      state.pendingBalances[payload.account] = payload.balances
    })
})
