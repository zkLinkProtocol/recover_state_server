import { useDispatch, useSelector } from 'react-redux'
import { useWeb3React } from '@web3-react/core'
import { useCallback, useMemo } from 'react'
import { connectorByName, ConnectorNames } from '../../connectors'
import { updateConnectorName } from './actions'
import { RootState } from '..'
import {
  Balance,
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
import { TESTER } from '../../config'

export function isStableCoin(tokenId: number) {
  return tokenId >= 17 && tokenId <= 31
}
export const useCurrentAccount = () => {
  return useSelector<RootState, Address>((state) => state.home.account)
}
export const useNetworks = () => {
  return useSelector<RootState, NetworkInfo[]>((state) => state.home.networks)
}
export const useCurrentChain = () => {
  return useSelector<RootState, NetworkInfo | undefined>((state) => state.home.currentChain)
}
export const useExodusMode = (chainId?: L2ChainId) => {
  const exodusMode = useSelector<RootState, { [x: L2ChainId]: number }>(
    (state) => state.home.exodusMode
  )
  if (!chainId) {
    return undefined
  }
  return exodusMode[chainId]
}
export const useTotalBlocksExecuted = (chainId?: L2ChainId) => {
  const useTotalBlocksExecuted = useSelector<RootState, { [x: L2ChainId]: number }>(
    (state) => state.home.totalBlocksExecuted
  )
  if (!chainId) {
    return undefined
  }
  return useTotalBlocksExecuted[chainId]
}
export const useRecoverProgress = () => {
  return useSelector<RootState, RecoverProgress | undefined>((state) => state.home.recoverProgress)
}
export const useRecoverMaxBlock = () => {
  const progress = useRecoverProgress()
  const currentChain = useCurrentChain()
  const totalBlocksExecuted = useTotalBlocksExecuted(currentChain?.layerTwoChainId)
  return TESTER ? progress?.total_verified_block : totalBlocksExecuted
}
export const useRecoverProgressCompleted = () => {
  const progress = useRecoverProgress()
  const maxBlock = useRecoverMaxBlock()

  if (progress && maxBlock) {
    return progress.current_block >= maxBlock
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
export const useProofTokens = (tokenId: TokenId) => {
  const tokens = useTokens()

  return useMemo(() => {
    const r = []
    if (tokenId === 1) {
      for (let i in tokens) {
        if (isStableCoin(Number(i))) {
          r.push(tokens[i])
        }
      }
    } else {
      r.push(tokens[tokenId])
    }
    return r
  }, [tokens, tokenId])
}
export const useContracts = () => {
  return useSelector<RootState, Contracts | undefined>((state) => state.home.contracts)
}
export const useConnectorName = () => {
  return useSelector<RootState, ConnectorNames | undefined>((state) => state.home.connectorName)
}
export const useBalance = () => {
  return useSelector<RootState, Balance>((state) => state.home.balance)
}
export const useMulticallContracts = () => {
  return useSelector<RootState, string[] | undefined>((state) => state.home.multicallContracts)
}
export const usePendingBalance = (account?: Address) => {
  const balance = useSelector<RootState, RootState['home']['pendingBalance']>(
    (state) => state.home.pendingBalance
  )
  if (!account) {
    return undefined
  }
  return balance[account]
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
export const useProofByToken = (args: {
  chainId: L2ChainId
  accountAddress: Address
  subAccountId: SubAccountId
  l2SourceToken: TokenId
  l1TargetToken: TokenId
}) => {
  const proofs = useSelector<RootState, HomeState['proofs']>((state) => state.home.proofs)

  return proofs?.find(
    (v) =>
      v.chainId === args.chainId &&
      v.accountAddress === args.accountAddress &&
      v.subAccountId === args.subAccountId &&
      v.l2SourceToken === args.l2SourceToken &&
      v.l1TargetToken === args.l1TargetToken
  )?.proof
}
// export const useCurrentChainProofs = (subAccountId: SubAccountId, tokenId: TokenId) => {
//   const proofs = useProofs(subAccountId, tokenId)
//   const currentChain = useCurrentChain()
//   return proofs?.filter((v) => v.exit_info.chain_id === currentChain?.layerTwoChainId)
// }

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
