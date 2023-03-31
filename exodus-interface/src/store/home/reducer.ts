import { createReducer } from '@reduxjs/toolkit'
import {
  fetchMulticallContracts,
  fetchProofs,
  updateBalances,
  updateContracts,
  updateCurrentChain,
  updateProofs,
  updateStoredBlockInfo,
  updateTokens,
} from './actions'
import { chainList } from '../../config/chains'
import { HomeState } from './types'
import { http } from '../../api'

const initialState: HomeState = {
  currentChain: undefined,
  contracts: {},
  connectorName: undefined,
  tokens: {},
  balances: {},
  storedBlockInfos: {},
  proofs: {},
  multicallContracts: undefined,
}

export default createReducer<HomeState>(initialState, (builder) => {
  builder
    .addCase(updateCurrentChain, (state, { payload }) => {
      state.currentChain = payload
    })
    .addCase(updateContracts, (state, { payload }) => {
      if (!payload) {
        return
      }
      if (!state.currentChain) {
        const chainId = Number(Object.keys(payload)[0])
        state.currentChain = chainList[chainId]
      }
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
    .addCase(fetchMulticallContracts.fulfilled, (state, { payload }) => {
      state.multicallContracts = payload
    })
})
