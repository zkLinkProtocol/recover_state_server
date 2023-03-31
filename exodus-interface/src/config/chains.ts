import { JsonRpcProvider } from '@ethersproject/providers'
import { L2ChainId } from '../types/global'

export interface ChainInfo {
  name: string
  chainId: number
  symbol: string
  decimals: number
  rpcUrl: string
  explorerUrl: string
  l2ChainId: number
}
export const chainList: {
  [x: L2ChainId]: ChainInfo
} = {
  1: {
    name: 'Polygon Testnet',
    chainId: 80001,
    symbol: 'MATIC',
    decimals: 18,
    rpcUrl: 'https://matic-mumbai.chainstacklabs.com',
    explorerUrl: 'https://explorer-mumbai.maticvigil.com',
    l2ChainId: 1,
  },
  2: {
    name: 'AVAX Testnet',
    chainId: 43113,
    symbol: 'MATIC',
    decimals: 18,
    rpcUrl: 'https://api.avax-test.network/ext/bc/C/rpc',
    explorerUrl: 'https://testnet.snowtrace.io',
    l2ChainId: 2,
  },
}

const providers: {
  [x: L2ChainId]: JsonRpcProvider
} = {}
for (let i in chainList) {
  providers[i] = new JsonRpcProvider(chainList[i].rpcUrl)
}

export function providerByChainId(chainId: L2ChainId) {
  return providers[chainId]
}
