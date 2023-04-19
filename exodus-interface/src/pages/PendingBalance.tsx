import { Button, CircularProgress, Stack, Typography, styled } from '@mui/material'
import { Section } from './L2Balances'
import {
  useContracts,
  useCurrentAccount,
  useCurrentChain,
  usePendingBalances,
  useTokens,
} from '../store/home/hooks'
import { useWeb3React } from '@web3-react/core'
import { Interface, formatEther } from 'ethers/lib/utils'
import MainContract from 'zklink-js-sdk/abi/ZkLink.json'
import { Wei } from '../types/global'
import { BigNumber } from 'ethers'
import { useEffectOnce, useInterval } from 'usehooks-ts'
import { useDispatch } from 'react-redux'
import { useAppDispatch } from '../store'
import { fetchPendingBalances } from '../store/home/actions'
import { TokenIcon } from '../components/Icon'
import { PendingBalance } from '../store/home/types'
import { FC, useEffect, useState } from 'react'
import toast from 'react-hot-toast'
import { PENDING_BALANCE_DELAY } from '../config'

const ColumnToken = styled(Stack)(({ theme }) => ({
  display: 'flex',
  alignItems: 'center',
  flexDirection: 'row',
}))
const ColumnBalance = styled('div')({
  textAlign: 'right',
  flex: 1,
})

const BalanceRowWrap = styled(Stack)(({ theme }) => ({
  fontSize: 18,
  padding: '16px 0',
  alignItems: 'center',
  flexDirection: 'row',
  [theme.breakpoints.down('md')]: {
    flexDirection: 'column',
  },
  borderBottom: '0.5px solid #aaa',
  transition: 'background .2s ease',
  '&:hover': {
    backgroundColor: 'rgba(0, 0, 0, 0.05)',
  },
  '>:not(style)+:not(style)': {
    marginLeft: 80,
    [theme.breakpoints.down('md')]: {
      marginTop: 16,
      marginLeft: 0,
    },
  },
}))

const BalanceRowToken = styled(Stack)(({ theme }) => ({
  flex: 1,
  alignItems: 'center',
  flexDirection: 'row',
  width: 'auto',
  [theme.breakpoints.down('md')]: {
    width: '100%',
  },
}))
const BalanceRowAction = styled(Stack)(({ theme }) => ({
  flex: 1,
  textAlign: 'right',
  justifyContent: 'flex-end',
}))

export const SectionPendingBalance = () => {
  const { provider, account } = useWeb3React()
  const dispatch = useAppDispatch()
  const pendingBalances = usePendingBalances(account)

  const requestBalance = () => {
    if (!provider || !account) {
      return
    }

    return dispatch(
      fetchPendingBalances({
        provider,
        account,
      })
    )
  }
  useEffect(() => {
    requestBalance()
  }, [account])
  useInterval(() => {
    requestBalance()
  }, PENDING_BALANCE_DELAY)

  return (
    <Section
      sx={{
        mb: 4,
      }}
    >
      <Typography variant="h5">Pending Balances</Typography>
      <Typography sx={{ fontStyle: 'italic' }} color="gray" variant="body1">
        After sending the Layer2 withdrawal request, you can harvest tokens to your wallet.
      </Typography>

      {pendingBalances?.length ? (
        <BalanceRowWrap
          sx={{
            fontSize: 14,
            color: 'gray',
          }}
        >
          <BalanceRowToken>
            <ColumnToken>Token</ColumnToken>
            <ColumnBalance>Balance</ColumnBalance>
          </BalanceRowToken>
          <BalanceRowAction>Action</BalanceRowAction>
        </BalanceRowWrap>
      ) : null}

      {pendingBalances ? (
        pendingBalances.length ? (
          pendingBalances.map((item) => <PendingBalanceRow key={item.token_id} item={item} />)
        ) : (
          <Typography
            sx={{
              textAlign: 'center',
              p: 5,
            }}
          >
            No Balance
          </Typography>
        )
      ) : (
        <Stack
          sx={{
            width: '100%',
            p: 5,
          }}
          alignItems="center"
        >
          <CircularProgress sx={{ mr: 0.5 }} color="success" size={24} />
        </Stack>
      )}
    </Section>
  )
}

export const PendingBalanceRow: FC<{ item: PendingBalance }> = ({ item }) => {
  const { provider, account } = useWeb3React()
  const contracts = useContracts()
  const currentChain = useCurrentChain()
  const [pending, setPending] = useState(false)
  return (
    <BalanceRowWrap>
      <BalanceRowToken>
        <ColumnToken>
          <Stack direction="row" alignItems="center" spacing={0.5}>
            <TokenIcon symbol={item.symbol} size={18} />
            <span>{item.symbol}</span>
          </Stack>
        </ColumnToken>
        <ColumnBalance>{formatEther(item.balance)}</ColumnBalance>
      </BalanceRowToken>
      <BalanceRowAction>
        <Button
          sx={{
            fontSize: 16,
            textTransform: 'none',
            pt: 0,
            pb: 0,
          }}
          color="success"
          variant="text"
          fullWidth={true}
          onClick={async () => {
            try {
              if (!provider || !contracts || !currentChain || pending) {
                return
              }
              setPending(true)
              const iface = new Interface(MainContract.abi)
              const fragment = iface.getFunction('withdrawPendingBalance')
              const calldata = iface.encodeFunctionData(fragment, [
                account,
                item.token_id,
                item.balance,
              ])
              const tx = await provider.send('eth_sendTransaction', [
                {
                  from: account,
                  to: contracts[currentChain.layerTwoChainId],
                  data: calldata,
                },
              ])
              if (tx) {
                toast.success(
                  (t) => (
                    <Stack>
                      <Typography>Transaction sent successfully</Typography>
                      <Button
                        onClick={() => {
                          window.open(currentChain.explorerUrl + '/tx/' + tx)
                        }}
                      >
                        View On Explorer
                      </Button>
                    </Stack>
                  ),
                  {
                    duration: 5000,
                  }
                )
              }
            } catch (e: any) {
              toast.error(e?.message)
              console.log(e)
            }
            setPending(false)
          }}
        >
          {pending ? <CircularProgress sx={{ mr: 0.5 }} color="success" size={14} /> : null}
          <span>Harvest</span>
        </Button>
      </BalanceRowAction>
    </BalanceRowWrap>
  )
}