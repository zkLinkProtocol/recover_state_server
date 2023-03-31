import axios from 'axios'
import { useEffect } from 'react'
import { useDispatch } from 'react-redux'
import {
  updateBalances,
  updateContracts,
  updateCurrentChain,
  updateStoredBlockInfo,
  updateTokens,
} from './actions'
import { useWeb3React } from '@web3-react/core'
import { connectorByName, ConnectorNames } from '../../connectors'
import { http } from '../../api'
import { chainList, providerByChainId } from '../../config/chains'
import { Contract } from '@ethersproject/contracts'
import { Interface } from '@ethersproject/abi'
import { useMulticallContracts } from './hooks'

export const Updater = () => {
  const dispatch = useDispatch()
  const { account, isActive, chainId } = useWeb3React()
  useEffect(() => {
    http.get('/contracts').then((r) => {
      const { data } = r.data
      dispatch(updateContracts(data))
    })
  }, [])

  useEffect(() => {
    http.get('/tokens').then(async (r) => {
      const { data } = r.data
      // for (let tokenId in data) {
      //   if (tokenId == '1') {
      //     data[tokenId].symbol = 'USD'
      //     continue
      //   }
      //   for (let chainId in data[tokenId].addresses) {
      //     const address = data[tokenId].addresses[chainId]
      //     if (address) {
      //       try {
      //         const p = providerByChainId(chainId)
      //         const iface = new Interface(['function symbol() view returns (string)'])
      //         const fragment = iface.getFunction('symbol')
      //         const calldata = iface.encodeFunctionData(fragment!, [])
      //         const resultData = await p.call({ to: address, data: calldata })
      //         if (resultData === '0x') {
      //           continue
      //         }
      //         const symbol = iface.decodeFunctionResult(fragment!, resultData)
      //         data[tokenId].symbol = symbol[0]
      //         break
      //       } catch (e) {
      //         console.log(e)
      //       }
      //     }
      //   }
      // }
      dispatch(updateTokens(data))
    })
  }, [])

  useEffect(() => {
    if (account) {
      http
        .post('/get_balances', {
          address: account,
        })
        .then((r) => {
          const { data } = r.data
          // const balances = []
          // for (let i in r.data) {
          //   for (let t in r.data[i]) {
          //     balances.push({
          //       subAccountId: Number(i),
          //       tokenId: Number(t),
          //       balance: r.data[i][t],
          //     })
          //   }
          // }
          dispatch(updateBalances(data))
        })
    }
  }, [account])

  useEffect(() => {
    if (!isActive) {
      connectorByName(ConnectorNames.Metamask).connectEagerly()
    }
  }, [isActive])

  useEffect(() => {
    try {
      if (!chainId) {
        throw new Error('Unknown Network')
      }
      const currentChain = Object.values(chainList)?.find((v) => v.chainId === chainId)
      if (!currentChain) {
        throw new Error('Unknown Network')
      }
      dispatch(updateCurrentChain(currentChain))
      http
        .post('/get_stored_block_info', {
          chain_id: currentChain?.l2ChainId,
        })
        .then((r) => {
          dispatch(
            updateStoredBlockInfo({
              chainId: currentChain?.l2ChainId,
              storedBlockInfo: r.data.data,
            })
          )
        })
    } catch (e) {
      dispatch(updateCurrentChain(undefined))
    }
  }, [chainId])

  return null
}
