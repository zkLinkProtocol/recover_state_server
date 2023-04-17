import { useDispatch, useSelector } from 'react-redux'
import { useWeb3React } from '@web3-react/core'
import { useCallback, useMemo } from 'react'
import { connectorByName, ConnectorNames } from '../../connectors'
import { updateConnectorName } from './actions'
import { RootState } from '..'
import { Balances, Contracts, HomeState, Tokens } from './types'
import { ChainInfo, chainList } from '../../config/chains'
import { ChainId, L2ChainId, SubAccountId, TokenId } from '../../types/global'
import { Web3Provider } from '@ethersproject/providers'

export const useCurrentChain = () => {
  return useSelector<RootState, ChainInfo | undefined>((state) => state.home.currentChain)
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
          const network = Object.values(chainList).find((v) => v.chainId === chainId)
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
    [provider]
  )
}
