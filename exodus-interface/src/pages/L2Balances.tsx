import { Typography, Stack, Box } from '@mui/material'
import { styled } from '@mui/system'
import { FC, useEffect, useState, useCallback } from 'react'
import {
  useBalances,
  useContracts,
  useCurrentChain,
  useProofs,
  useStoredBlockInfo,
  useTokens,
} from '../store/home/hooks'
import { Ether, Wei } from '../types/global'
import { http } from '../api'
import { useWeb3React } from '@web3-react/core'
import { toast } from 'react-toastify'
import { ProofInfo } from '../store/home/types'
import { fetchProofs } from '../store/home/actions'
import { useAppDispatch } from '../store'
import { TokenIcon } from '../components/Icon'
import { Interface } from '@ethersproject/abi'
import { formatEther } from '@ethersproject/units'
import MainContract from 'zklink-js-sdk/abi/ZkLink.json'
import { chainList } from '../config/chains'
import * as mathjs from 'mathjs'
import CircularProgress from '@mui/material/CircularProgress'
import Button from '@mui/material/Button'
import { SyncBlock } from './SyncBlock'
import { Header } from './Header'

export const Section = styled(Box)({
  backgroundColor: 'rgba(237, 237, 237)',
  padding: 16,
  marginBottom: 16,
  boxShadow: '4px 4px 0 rgb(218, 218, 218)',
})

const ColumnToken = styled(Stack)(({ theme }) => ({
  display: 'flex',
  alignItems: 'center',
  flexDirection: 'row',
  width: 100,
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
const BalanceRowProof = styled(Stack)(({ theme }) => ({
  flex: 1,
  textAlign: 'right',
  [theme.breakpoints.down('md')]: {
    width: '100%',
  },
}))
export const L2Balances = () => {
  const balances = useBalances()

  const renderList = () => {
    const list = []
    for (let i in balances) {
      list.push(<BalanceList key={i} subAccountId={Number(i)} list={balances[i]} />)
    }

    return list
  }
  const list = renderList()
  return (
    <>
      <Header />

      <Section
        sx={{
          mb: 1,
        }}
      >
        <Typography variant="h5">Layer2 Balances</Typography>
        <Typography sx={{ fontStyle: 'italic' }} color="gray" variant="body1">
          Step 1: Connect your wallet to check your balance.
          <br />
          Step 2: Generate proofs for each token.
          <br />
          Step 3: Send a withdrawal transaction to withdraw the tokens to your wallet.
          <br />
          Step 4: Repeat the above steps for the other chains.
        </Typography>
        <SyncBlock />
        {list?.length ? (
          list
        ) : (
          <Typography
            sx={{
              textAlign: 'center',
              padding: '64px 0',
            }}
            color="gray"
          >
            No Balances
          </Typography>
        )}
      </Section>
    </>
  )
}

const BalanceList: FC<any> = ({ subAccountId, list }) => {
  const renderRows = () => {
    const rows = []
    for (let i in list) {
      rows.push(
        <BalanceRow
          key={i}
          item={{
            subAccountId,
            tokenId: Number(i),
            balance: list[i],
          }}
        />
      )
    }

    return rows
  }

  return (
    <Box sx={{ mt: 4 }}>
      <Typography variant="subtitle1" sx={{ mt: 2 }}>
        Account {subAccountId}
      </Typography>

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
        <BalanceRowProof>Proofs / Withdrawal</BalanceRowProof>
      </BalanceRowWrap>
      {renderRows()}
    </Box>
  )
}

const BalanceRow: FC<{
  item: {
    subAccountId: number
    tokenId: number
    balance: Wei
  }
}> = ({ item }) => {
  const dispatch = useAppDispatch()
  const tokens = useTokens()
  const symbol = tokens[item.tokenId]?.symbol
  const { account } = useWeb3React()
  const proofs = useProofs(item.subAccountId, item.tokenId)
  const [pending, setPending] = useState(false)

  const getProofs = useCallback(() => {
    if (!account || item?.subAccountId === undefined || item?.tokenId === undefined) {
      return
    }

    dispatch(
      fetchProofs({
        address: account,
        sub_account_id: item.subAccountId,
        token_id: item.tokenId,
      })
    )
  }, [account, item.subAccountId, item.tokenId])
  useEffect(() => {
    const t = setInterval(() => {
      getProofs()
    }, 30000)
    getProofs()
    return () => clearInterval(t)
  }, [getProofs])

  return (
    <BalanceRowWrap>
      <BalanceRowToken>
        <ColumnToken>
          <TokenIcon symbol={symbol} />
          <Typography sx={{ ml: 1, fontSize: 18 }}>{symbol || item.tokenId}</Typography>
        </ColumnToken>
        <ColumnBalance>{Number(formatEther(item.balance)).toFixed(18)}</ColumnBalance>
      </BalanceRowToken>

      <BalanceRowProof>
        {proofs?.length ? (
          <Proofs proofs={proofs} />
        ) : (
          <Typography
            sx={{
              fontSize: 18,
              fontWeight: 500,
            }}
            color="primary"
          >
            <Button
              sx={{
                fontSize: 16,
                textTransform: 'none',
                pt: 0,
                pb: 0,
              }}
              variant="text"
              onClick={async () => {
                try {
                  if (pending) {
                    return
                  }
                  setPending(true)
                  const tasks = await http.post('/generate_proof_tasks_by_token', {
                    address: account,
                    sub_account_id: item.subAccountId,
                    token_id: item.tokenId,
                  })
                  setPending(false)

                  if (tasks.data?.err_msg) {
                    toast.error(tasks.data?.err_msg, {
                      autoClose: 5000,
                    })
                  } else if (tasks.data?.code === 0) {
                    toast.success('Generate success', {})

                    dispatch(
                      fetchProofs({
                        address: account!,
                        sub_account_id: item.subAccountId,
                        token_id: item.tokenId,
                      })
                    )
                  }
                } catch (e) {
                  setPending(false)
                }
              }}
            >
              {pending ? <CircularProgress sx={{ mr: 0.5 }} size={14} /> : null}
              <span>Generate</span>
            </Button>
          </Typography>
        )}
      </BalanceRowProof>
    </BalanceRowWrap>
  )
}

function toFixed(ether: Ether) {
  let price = mathjs.multiply(mathjs.bignumber(ether), 1000000)
  price = mathjs.floor(Number(price))
  price = mathjs.divide(price, 1000000)
  return price
}

const Proofs: FC<{ proofs: ProofInfo[] }> = ({ proofs }) => {
  const tokens = useTokens()
  const currentChain = useCurrentChain()
  const contracts = useContracts()
  const storedBlockInfo = useStoredBlockInfo(currentChain?.l2ChainId)
  const { provider, account } = useWeb3React()
  return (
    <Stack spacing={0.5} width="100%">
      {proofs?.map((proofInfo, index) => (
        <Stack width="100%" key={index} flex="1" direction="row" justifyContent="space-between">
          <Stack direction="row" alignItems="center" spacing={0.5}>
            <TokenIcon symbol={tokens[proofInfo.exit_info.l1_target_token]?.symbol} size={18} />
            <Typography variant="body1">
              {tokens[proofInfo.exit_info.l1_target_token]?.symbol}:
            </Typography>
            <Typography variant="body2" color="GrayText">
              {proofInfo.amount !== null ? toFixed(formatEther(proofInfo.amount)) : '-'}
            </Typography>
          </Stack>
          {proofInfo.proof ? (
            proofInfo.exit_info.chain_id === currentChain?.l2ChainId ? (
              <Typography
                sx={(theme) => ({
                  color: theme.palette.success.main,
                  cursor: 'pointer',
                  fontWeight: 500,
                  fontSize: 18,
                })}
                variant="body1"
              >
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
                    if (!provider || !contracts || !currentChain || !proofInfo?.proof) {
                      return
                    }
                    const payload = [
                      {
                        blockNumber: storedBlockInfo?.block_number,
                        priorityOperations: storedBlockInfo?.priority_operations,
                        pendingOnchainOperationsHash:
                          storedBlockInfo?.pending_onchain_operations_hash,
                        timestamp: storedBlockInfo?.timestamp,
                        stateHash: storedBlockInfo?.state_hash,
                        commitment: storedBlockInfo?.commitment,
                        syncHash: storedBlockInfo?.sync_hash,
                      },
                      account,
                      proofInfo.exit_info.account_id,
                      proofInfo.exit_info.sub_account_id,
                      proofInfo.exit_info.l1_target_token,
                      proofInfo.exit_info.l2_source_token,
                      proofInfo.amount,
                      proofInfo.proof.proof,
                    ]
                    const iface = new Interface(MainContract.abi)
                    const fragment = iface.getFunction('performExodus')
                    const calldata = iface.encodeFunctionData(fragment, payload)
                    const tx = await provider.send('eth_sendTransaction', [
                      {
                        from: account,
                        to: contracts[currentChain.l2ChainId],
                        data: calldata,
                      },
                    ])
                    console.log(tx)
                  }}
                >
                  Withdraw
                </Button>
              </Typography>
            ) : (
              <Typography color="gray" sx={{ fontSize: 14 }}>
                On {chainList[proofInfo.exit_info.chain_id].name}
              </Typography>
            )
          ) : (
            <Typography
              sx={(theme) => ({
                color: theme.palette.info.main,
              })}
            >
              <Button
                sx={{
                  fontSize: 16,
                  textTransform: 'none',
                  pt: 0,
                  pb: 0,
                }}
                variant="text"
              >
                <CircularProgress sx={{ mr: 0.5 }} size={14} />
                <span>Generating</span>
              </Button>
            </Typography>
          )}
        </Stack>
      ))}
    </Stack>
  )
}
