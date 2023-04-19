import { useDispatch, useSelector } from 'react-redux'
import { useWeb3React } from '@web3-react/core'
import { useCallback, useMemo } from 'react'
import { connectorByName, ConnectorNames } from '../../connectors'
import { updateConnectorName } from './actions'
import { RootState } from '..'
import {
  Balances,
  Contracts,
  HomeState,
  NetworkInfo,
  PendingBalance,
  ProofHistory,
  RecoverProgress,
  Tokens,
} from './types'
import { ChainId, L2ChainId, SubAccountId, TokenId } from '../../types/global'
import { Web3Provider } from '@ethersproject/providers'
import { Address } from 'zklink-js-sdk/build/types'

export const useCurrentAccount = () => {
  return useSelector<RootState, Address>((state) => state.home.account)
}
export const useNetworks = () => {
  return useSelector<RootState, NetworkInfo[]>((state) => state.home.networks)
}
export const useCurrentChain = () => {
  return useSelector<RootState, NetworkInfo | undefined>((state) => state.home.currentChain)
}
export const useRecoverProgress = () => {
  return useSelector<RootState, RecoverProgress | undefined>((state) => state.home.recoverProgress)
}
export const useRecoverProgressCompleted = () => {
  const progress = useRecoverProgress()
  if (progress) {
    return progress.current_block >= progress.total_verified_block
  } else {
    return undefined
  }
}
export const useRunningTaskId = () => {
  return useSelector<RootState, number>((state) => state.home.runningTaskId)
}
export const useTokens = () => {
  return useSelector<RootState, Tokens>((state) => state.home.tokens)
}
export const useContracts = () => {
  return useSelector<RootState, Contracts | undefined>((state) => state.home.contracts)
}
export const useConnectorName = () => {
  return useSelector<RootState, ConnectorNames | undefined>((state) => state.home.connectorName)
}
export const useBalances = () => {
  return useSelector<RootState, Balances>((state) => state.home.balances)
}
export const useMulticallContracts = () => {
  return useSelector<RootState, string[] | undefined>((state) => state.home.multicallContracts)
}
export const usePendingBalances = (account?: Address) => {
  const balances = useSelector<RootState, RootState['home']['pendingBalances']>(
    (state) => state.home.pendingBalances
  )
  if (!account) {
    return undefined
  }
  return balances[account]
}

export const useStoredBlockInfo = (chainId?: L2ChainId) => {
  const storedBlockInfos = useSelector<RootState, HomeState['storedBlockInfos']>(
    (state) => state.home.storedBlockInfos
  )
  if (!chainId) {
    return undefined
  }
  return storedBlockInfos[chainId]
}
export const useProofs = (subAccountId: SubAccountId, tokenId: TokenId) => {
  const proofs = useSelector<RootState, HomeState['proofs']>((state) => state.home.proofs)
  if (proofs[subAccountId]) {
    return proofs[subAccountId][tokenId]
  }
  return undefined
}

export const useConnectWallet = () => {
  const dispatch = useDispatch()
  const { provider, isActive } = useWeb3React()
  return useCallback(async (connectorName: ConnectorNames) => {
    try {
      await connectorByName(connectorName).activate()
      dispatch(updateConnectorName(connectorName))
      return Promise.resolve()
    } catch (e) {
      return Promise.reject()
    }
  }, [])
}

export function useSwitchNetwork() {
  const { provider } = useWeb3React<Web3Provider>()
  const networks = useNetworks()
  return useCallback(
    async (chainId: ChainId) => {
      const chainIdString = `0x${Number(chainId).toString(16)}`
      if (!provider) {
        return
      }
      try {
        await provider.send('wallet_switchEthereumChain', [{ chainId: chainIdString }])
      } catch (e: any) {
        if (e?.code === 4902) {
          const network = networks.find((v) => v.chainId === chainId)
          if (!network) {
            return
          }
          await provider
            .send('wallet_addEthereumChain', [
              {
                chainId: chainIdString,
                chainName: network.name as string,
                nativeCurrency: {
                  name: network.symbol,
                  symbol: network.symbol,
                  decimals: network.decimals,
                },
                rpcUrls: [network.rpcUrl],
                blockExplorerUrls: [network.explorerUrl],
                iconUrls: [],
              },
            ])
            .catch(
              (
                e: Error & {
                  code: number
                }
              ) => {
                console.error(e)
              }
            )
        } else {
          console.log(e)
        }
      }
    },
    [provider, networks]
  )
}

export const useProofHistory = () => {
  return useSelector<RootState, ProofHistory | undefined>((state) => state.home.proofHistory)
}
