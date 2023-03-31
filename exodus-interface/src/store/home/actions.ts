import { createAction, createAsyncThunk } from '@reduxjs/toolkit'
import { Tokens, HomeState, Balances, StoredBlockInfo, ProofInfo } from './types'
import { ChainInfo } from '../../config/chains'
import { Address, L2ChainId, SubAccountId, TokenId } from '../../types/global'
import { http } from '../../api'
import axios, { AxiosResponse } from 'axios'
import { GITHUB_STATIC_PATH } from '../../config'
import { useWeb3React } from '@web3-react/core'

export const updateCurrentChain = createAction<ChainInfo | undefined>('home/updateCurrentChain')
export const updateConnectorName = createAction<string>('home/updateConnectorName')
export const updateContracts = createAction<HomeState['contracts']>('home/updateContracts')
export const updateTokens = createAction<Tokens>('home/updateTokens')
export const updateBalances = createAction<Balances>('home/updateBalances')
export const updateStoredBlockInfo = createAction<{
  chainId: L2ChainId
  storedBlockInfo: StoredBlockInfo
}>('home/updateStoredBlockInfo')
export const updateProofs = createAction<{
  address: Address
  sub_account_id: number
  token_id: number
}>('home/updateProofs')

interface ProofsArgs {
  address: Address
  sub_account_id: number
  token_id: number
}
export const fetchProofs = createAsyncThunk<
  {
    subAccountId: SubAccountId
    tokenId: TokenId
    data: ProofInfo[]
  },
  ProofsArgs
>('home/fetchProofs', async (args) => {
  const r = await http.post('/get_proofs_by_token', {
    address: args.address,
    sub_account_id: args.sub_account_id,
    token_id: args.token_id,
  })

  return {
    subAccountId: args.sub_account_id,
    tokenId: args.token_id,
    data: r.data.data,
  }
})
export const fetchMulticallContracts = createAsyncThunk<string[]>(
  'home/fetchMulticallContracts',
  async () => {
    const r = await axios.get(`${GITHUB_STATIC_PATH}/contracts/main.json`)
    console.log(r)
    return r.data
  }
)
// export const withdraw = createAsyncThunk('home/withdraw', async () => {
//   const abis = [
//     'function performExodus(StoredBlockInfo calldata _storedBlockInfo, address _owner, uint32 _accountId, uint8 _subAccountId, uint16 _withdrawTokenId, uint16 _deductTokenId, uint128 _amount, uint256[] calldata _proof) external notActive nonReentrant',
//   ]
// })
