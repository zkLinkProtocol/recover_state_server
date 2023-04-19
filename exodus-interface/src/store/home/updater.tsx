import { useEffect } from 'react'
import {
  fetchNetworks,
  fetchRecoverProgress,
  fetchRunningTaskId,
  updateBalances,
  updateContracts,
  updateCurrentChain,
  updateStoredBlockInfo,
  updateTokens,
} from './actions'
import { useWeb3React } from '@web3-react/core'
import { connectorByName, ConnectorNames } from '../../connectors'
import { http } from '../../api'
import { useCurrentChain, useNetworks, useRecoverProgressCompleted } from './hooks'
import { useEffectOnce, useInterval } from 'usehooks-ts'
import { useAppDispatch } from '..'
import { RECOVER_PROGRESS_DELAY, RUNNING_TASK_ID_DELAY } from '../../config'

export const useFetchRecoverProgress = () => {
  const recoverProgressCompleted = useRecoverProgressCompleted()
  const dispatch = useAppDispatch()
  useEffectOnce(() => {
    dispatch(fetchRecoverProgress())
  })

  useInterval(
    () => {
      if (!recoverProgressCompleted) {
        dispatch(fetchRecoverProgress())
      }
    },
    recoverProgressCompleted ? null : RECOVER_PROGRESS_DELAY
  )
}

export const useFetchRunningTaskId = () => {
  const recoverProgressCompleted = useRecoverProgressCompleted()
  const dispatch = useAppDispatch()
  useEffect(() => {
    if (!recoverProgressCompleted) {
      return
    }
    dispatch(fetchRunningTaskId())
  }, [recoverProgressCompleted])

  useInterval(
    () => {
      dispatch(fetchRunningTaskId())
    },
    recoverProgressCompleted ? RUNNING_TASK_ID_DELAY : null
  )
}

export const Updater = () => {
  const dispatch = useAppDispatch()
  const { account, isActive, chainId } = useWeb3React()
  const networks = useNetworks()
  const recoverProgressCompleted = useRecoverProgressCompleted()
  const currentChain = useCurrentChain()

  useFetchRecoverProgress()

  useFetchRunningTaskId()

  useEffectOnce(() => {
    dispatch(fetchNetworks())
  })

  useEffectOnce(() => {
    http.get('/contracts').then((r) => {
      const { data } = r.data
      dispatch(updateContracts(data))
    })
  })

  useEffect(() => {
    if (recoverProgressCompleted) {
      http.get('/tokens').then((r) => {
        const { data } = r.data
        dispatch(updateTokens(data))
      })
    }
  }, [recoverProgressCompleted])

  useEffect(() => {
    if (!currentChain || !recoverProgressCompleted) {
      return
    }

    http
      .post('/get_stored_block_info', {
        chain_id: currentChain?.layerTwoChainId,
      })
      .then((r) => {
        dispatch(
          updateStoredBlockInfo({
            chainId: currentChain?.layerTwoChainId,
            storedBlockInfo: r.data.data,
          })
        )
      })
  }, [currentChain, recoverProgressCompleted])

  useEffect(() => {
    if (account && recoverProgressCompleted) {
      http
        .post('/get_balances', {
          address: account,
        })
        .then((r) => {
          const { data } = r.data
          dispatch(updateBalances(data))
        })
    }
  }, [account, recoverProgressCompleted])

  useEffect(() => {
    try {
      if (!chainId) {
        throw new Error('Unknown Network')
      }
      const currentChain = networks?.find((v) => v.chainId === chainId)
      if (!currentChain) {
        throw new Error('Unknown Network')
      }
      dispatch(updateCurrentChain(currentChain))
    } catch (e) {
      dispatch(updateCurrentChain(undefined))
    }
  }, [chainId, networks])

  useEffect(() => {
    if (!isActive) {
      connectorByName(ConnectorNames.Metamask).connectEagerly()
    }
  }, [isActive])

  return null
}
