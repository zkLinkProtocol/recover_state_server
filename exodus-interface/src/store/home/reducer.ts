import { createReducer } from '@reduxjs/toolkit'
import {
  fetchExodusMode,
  fetchNetworks,
  fetchPendingBalances,
  fetchProofHistory,
  fetchProofs,
  fetchRecoverProgress,
  fetchRunningTaskId,
  fetchTotalBlocksExecuted,
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
  exodusMode: {},
  totalBlocksExecuted: {},
  recoverProgress: undefined,
  runningTaskId: 0,
  connectorName: undefined,
  tokens: {},
  balance: {},
  storedBlockInfos: {},
  proofs: [],
  multicallContracts: undefined,
  proofHistory: undefined,
  pendingBalance: {},
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
    .addCase(fetchExodusMode.fulfilled, (state, { payload }) => {
      state.exodusMode[payload.chainId] = payload.exodusMode
    })
    .addCase(fetchTotalBlocksExecuted.fulfilled, (state, { payload }) => {
      state.totalBlocksExecuted[payload.chainId] = payload.totalBlocksExecuted
    })
    .addCase(updateTokens, (state, { payload }) => {
      state.tokens = payload
    })
    .addCase(updateBalances, (state, { payload }) => {
      state.balance = payload
    })
    .addCase(updateStoredBlockInfo, (state, { payload }) => {
      state.storedBlockInfos[payload.chainId] = payload.storedBlockInfo
    })
    .addCase(fetchProofs.fulfilled, (state, { payload }) => {
      const index = state.proofs.findIndex(
        (v) =>
          v.subAccountId === payload.subAccountId &&
          v.accountAddress === payload.accountAddress &&
          v.chainId === payload.chainId &&
          v.l1TargetToken === payload.l1TargetToken &&
          v.l2SourceToken === payload.l2SourceToken
      )
      if (index >= 0) {
        state.proofs[index]['proof'] = payload.proof
      } else {
        state.proofs.push(payload)
      }
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
      state.pendingBalance[payload.account] = payload.balances
    })
    .addCase(updatePendingBalances, (state, { payload }) => {
      state.pendingBalance[payload.account] = payload.balances
    })
})
