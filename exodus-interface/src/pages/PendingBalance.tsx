import { Button, CircularProgress, Stack, Typography, styled } from '@mui/material'
import { Section } from './L2Balance'
import {
  useContracts,
  useCurrentAccount,
  useCurrentChain,
  usePendingBalance,
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
import { FC, useCallback, useEffect, useState } from 'react'
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
const BalanceRowAction = styled('div')(({ theme }) => ({
  flex: 1,
  textAlign: 'right',
}))

export const SectionPendingBalance = () => {
  const { provider, chainId, account } = useWeb3React()
  const dispatch = useAppDispatch()
  const pendingBalances = usePendingBalance(account)

  const requestBalance = useCallback(() => {
    if (!provider || !account) {
      return
    }
    return dispatch(
      fetchPendingBalances({
        provider,
        account,
      })
    )
  }, [chainId, account])
  useEffect(() => {
    requestBalance()
  }, [requestBalance])
  useInterval(() => {
    requestBalance()
  }, PENDING_BALANCE_DELAY)

  return (
    <Section
      sx={{
        mb: 4,
      }}
    >
      <Typography variant="h5">Pending Balance</Typography>
      <Typography sx={{ fontStyle: 'italic' }} color="gray" variant="body1">
        Click on "Withdraw" and sign with your wallet to send the withdraw request on-chain. 
        <br/>
        This action can only be executed once, and once the withdraw transaction is confirmed on-chain, 
        this record will disappear. Please check your wallet to view the changes in your balance.
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

  const amounts = item.balance ? formatEther(item.balance)?.split('.') : formatEther('0')
  return (
    <BalanceRowWrap>
      <BalanceRowToken>
        <ColumnToken>
          <Stack direction="row" alignItems="center" spacing={0.5}>
            <TokenIcon symbol={item.symbol} size={18} />
            <span>{item.symbol}</span>
          </Stack>
        </ColumnToken>
        <ColumnBalance>
          <Stack direction="row" justifyContent="flex-end">
            {amounts[0] ? <Typography fontSize={18}>{amounts[0]}</Typography> : null}
            {amounts[1] ? (
              <Typography fontSize={18} color="gray">
                .{amounts[1]}
              </Typography>
            ) : null}
          </Stack>
        </ColumnBalance>
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
          <span>Withdraw</span>
        </Button>
      </BalanceRowAction>
    </BalanceRowWrap>
  )
}
