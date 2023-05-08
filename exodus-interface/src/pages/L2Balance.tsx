import { Typography, Stack, Box } from '@mui/material'
import { styled } from '@mui/system'
import { FC, useEffect, useState, useCallback, memo } from 'react'
import {
  useBalance,
  useContracts,
  useCurrentChain,
  useExodusMode,
  useNetworks,
  useRecoverProgressCompleted,
  useRunningTaskId,
  useStoredBlockInfo,
  useTokens,
  useProofTokens,
  useProofByToken,
} from '../store/home/hooks'
import { Ether, L2ChainId, TokenId, Wei } from '../types/global'
import { http } from '../api'
import { useWeb3React } from '@web3-react/core'
import { toast } from 'react-hot-toast'
import { ProofInfo, TokenInfo } from '../store/home/types'
import { fetchProofs } from '../store/home/actions'
import { useAppDispatch } from '../store'
import { TokenIcon } from '../components/Icon'
import { Interface } from '@ethersproject/abi'
import { formatEther } from '@ethersproject/units'
import MainContract from 'zklink-js-sdk/abi/ZkLink.json'
import * as mathjs from 'mathjs'
import CircularProgress from '@mui/material/CircularProgress'
import Button from '@mui/material/Button'
import { SyncBlock } from './SyncBlock'
import Verifier from '../abi/Verifier.json'
import { ethers } from 'ethers'
import { TESTER } from '../config'

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

interface L2TokenItem {
  subAccountId: number
  tokenId: number
  balance: Wei
}

export const SectionL2Balance = () => {
  const recoverProgressCompleted = useRecoverProgressCompleted()

  return (
    <>
      <Section>
        <Typography variant="h5">Layer2 Balance</Typography>
        <Typography sx={{ fontStyle: 'italic' }} color="gray" variant="body1">
          Step 1: Connect your wallet, and wait for the initialization to complete. You should see
          the balance of all your tokens on the webpage.
          <br />
          Step 2: Click on "Generate" button for each token, wait for your proof to be generated.
          Once a ZK-Proof is generated, the "Generate" button will change to "Submit" button.
          <br />
          Step 3: Click on "Submit", sign with your wallet to send the proof on-chain. Once the
          proof is verified on-chain, a list of withdrawable balances will appear in the
          PendingBalance. You’ll need to have the gas token of the destination blockchain to
          proceed. Also, please note that you only need to submit once, as the smart contract does
          not accept duplicate submissions.
          <br />
          Step 4: Now, click on "Withdraw", sign with your wallet to send the withdraw request
          on-chain.
          <br />
          Step 5: Switch network and repeat the above steps.
        </Typography>
        {recoverProgressCompleted ? <BalanceList /> : <SyncBlock />}
      </Section>
    </>
  )
}

const BalanceList = () => {
  const balances = useBalance()

  const renderList = () => {
    const list = []
    for (let i in balances) {
      list.push(<SubAccount key={i} subAccountId={Number(i)} list={balances[i]} />)
    }

    return list
  }
  const list = renderList()

  return (
    <>
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
    </>
  )
}

const SubAccount: FC<any> = ({ subAccountId, list }) => {
  const renderRows = () => {
    const rows = []
    for (let i in list) {
      rows.push(
        <BalanceRow
          key={i}
          l2Token={{
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
        <BalanceRowProof>Generate Proof / Submit Proof</BalanceRowProof>
      </BalanceRowWrap>
      {renderRows()}
    </Box>
  )
}

const BalanceRow: FC<{
  l2Token: L2TokenItem
}> = ({ l2Token }) => {
  const dispatch = useAppDispatch()
  const tokens = useTokens()
  const symbol = tokens[l2Token.tokenId]?.symbol
  const { account } = useWeb3React()

  const amounts = l2Token.balance ? formatEther(l2Token.balance)?.split('.') : formatEther('0')

  return (
    <BalanceRowWrap>
      <BalanceRowToken>
        <ColumnToken>
          <TokenIcon symbol={symbol} />
          <Typography sx={{ ml: 1, fontSize: 18 }}>{symbol || l2Token.tokenId}</Typography>
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

      <BalanceRowProof>
        <Proofs l2Token={l2Token} />
      </BalanceRowProof>
    </BalanceRowWrap>
  )
}

const Proofs: FC<{
  l2Token: L2TokenItem
}> = ({ l2Token }) => {
  const proofTokens = useProofTokens(l2Token.tokenId)
  return (
    <Stack spacing={0.5} width="100%">
      {proofTokens?.map((l1Token, index) => {
        const r = []
        for (let i in l1Token?.addresses) {
          r.push(
            <TokenProofRow
              key={`${l2Token.subAccountId}_${l2Token.tokenId}_${l1Token.token_id}_${i}`}
              l2Token={l2Token}
              layerTwoChainId={Number(i)}
              l1Token={l1Token}
            />
          )
        }
        return r
      })}
    </Stack>
  )
}

const TokenProofRow: FC<{
  l2Token: L2TokenItem
  l1Token: TokenInfo
  layerTwoChainId: L2ChainId
}> = memo(({ l2Token, l1Token, layerTwoChainId }) => {
  const networks = useNetworks()
  const { account } = useWeb3React()

  const proofInfo = useProofByToken({
    chainId: layerTwoChainId,
    accountAddress: account!,
    subAccountId: l2Token.subAccountId,
    l2SourceToken: l2Token.tokenId,
    l1TargetToken: l1Token.token_id,
  })
  const net = networks?.find((v) => v.layerTwoChainId === layerTwoChainId)
  const layerOneTokenAddress = l1Token?.addresses[layerTwoChainId]
  return (
    <Stack width="100%" flex="1" direction="row" alignItems="center" justifyContent="space-between">
      <Stack
        sx={{
          color: '#333',
          textDecoration: 'none',
          p: '0 4px',
          borderRadius: '2px',
          '&:hover': {
            backgroundColor: 'rgba(0, 0, 0, 0.05)',
          },
        }}
        component={'a'}
        href={`${net?.explorerUrl}/token/${layerOneTokenAddress}?a=${account}`}
        target="_blank"
        direction="row"
        alignItems="center"
        spacing={0.5}
      >
        <TokenIcon symbol={l1Token?.symbol} size={18} />
        <Typography variant="body1">{l1Token?.symbol}:</Typography>
        <Typography variant="body2" color="GrayText">
          {proofInfo &&
          proofInfo.proof_info?.amount !== null &&
          proofInfo.proof_info?.amount !== undefined
            ? formatEther(proofInfo.proof_info.amount)
            : '-'}
        </Typography>
      </Stack>
      <TokenProofAction
        l2Token={l2Token}
        l1Token={l1Token}
        layerTwoChainId={layerTwoChainId}
        proofInfo={proofInfo}
      />
    </Stack>
  )
})

const TokenProofAction: FC<{
  l2Token: L2TokenItem
  l1Token: TokenInfo
  layerTwoChainId: L2ChainId
  proofInfo?: ProofInfo
}> = memo(({ l2Token, l1Token, layerTwoChainId, proofInfo }) => {
  const dispatch = useAppDispatch()
  const currentChain = useCurrentChain()
  const contracts = useContracts()
  const storedBlockInfo = useStoredBlockInfo(currentChain?.layerTwoChainId)
  const { provider, account } = useWeb3React()
  const runningTaskId = useRunningTaskId()
  const networks = useNetworks()
  const [pending, setPending] = useState(false)
  const [verifyPending, setVerifyPending] = useState(false)
  const exodusMode = useExodusMode(currentChain?.layerTwoChainId)

  const getProofs = useCallback(() => {
    if (!currentChain?.layerTwoChainId) {
      return
    }

    if (
      !account ||
      l2Token?.subAccountId === undefined ||
      l2Token?.tokenId === undefined ||
      l1Token.token_id === undefined ||
      layerTwoChainId === undefined ||
      layerTwoChainId !== currentChain?.layerTwoChainId
    ) {
      return
    }

    dispatch(
      fetchProofs({
        chain_id: layerTwoChainId,
        account_address: account,
        sub_account_id: l2Token.subAccountId,
        l2_source_token: l2Token.tokenId,
        l1_target_token: l1Token.token_id,
      })
    )
  }, [
    account,
    l2Token.subAccountId,
    l2Token.tokenId,
    l1Token.token_id,
    layerTwoChainId,
    currentChain,
  ])

  useEffect(() => {
    const t = setInterval(() => {
      getProofs()
    }, 30000)
    getProofs()
    return () => clearInterval(t)
  }, [getProofs])

  if (currentChain?.layerTwoChainId !== layerTwoChainId) {
    const net = networks?.find((v) => v.layerTwoChainId === layerTwoChainId)
    return (
      <Typography color="gray" sx={{ fontSize: 14 }}>
        Switch to {net?.name}
      </Typography>
    )
  }
  if (!proofInfo) {
    return (
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
                throw new Error('')
              }
              if (TESTER === false && exodusMode !== 1) {
                throw new Error('Not exodus time')
              }
              setPending(true)
              const tasks = await http.post('/generate_proof_task_by_info', {
                chain_id: currentChain?.layerTwoChainId,
                account_address: account,
                account_id: 0,
                sub_account_id: l2Token.subAccountId,
                l1_target_token: l1Token.token_id,
                l2_source_token: l2Token.tokenId,
              })
              setPending(false)

              if (tasks.data?.err_msg) {
                throw new Error(tasks.data?.err_msg)
              } else if (tasks.data?.code === 0) {
                toast.success('Request sent successfully, waiting for generation')

                getProofs()
              }
            } catch (e: any) {
              if (e?.message) {
                toast.error(e?.message)
              }
              setPending(false)
            }
          }}
        >
          {pending ? <CircularProgress sx={{ mr: 0.5 }} size={14} /> : null}
          <span>Generate</span>
        </Button>
      </Typography>
    )
  }

  return proofInfo.proof_info?.proof ? (
    <Typography
      sx={(theme) => ({
        color: theme.palette.success.main,
        cursor: 'pointer',
        fontWeight: 500,
        fontSize: 18,
      })}
      variant="body1"
    >
      {TESTER ? (
        <Button
          sx={{
            fontSize: 16,
            textTransform: 'none',
            pt: 0,
            pb: 0,
          }}
          color="primary"
          variant="text"
          onClick={async () => {
            try {
              if (
                !provider ||
                !contracts ||
                !currentChain ||
                !proofInfo?.proof_info ||
                verifyPending
              ) {
                return
              }
              setVerifyPending(true)

              const mainIface = new Interface(MainContract.abi)
              const mainFragment = mainIface.getFunction('verifier')
              const mainCalldata = mainIface.encodeFunctionData(mainFragment, [])
              const verifyContractAddress = await provider.send('eth_call', [
                {
                  from: account,
                  to: contracts[currentChain.layerTwoChainId],
                  data: mainCalldata,
                },
              ])
              if (verifyContractAddress.length !== 66) {
                throw new Error('Invalid verifier contract address.')
              }
              const payload = [
                storedBlockInfo?.state_hash,
                proofInfo.exit_info.chain_id,
                proofInfo.exit_info.account_id,
                proofInfo.exit_info.sub_account_id,
                account,
                proofInfo.exit_info.l1_target_token,
                proofInfo.exit_info.l2_source_token,
                proofInfo.proof_info.amount,
                proofInfo.proof_info.proof?.proof,
              ]
              const iface = new Interface(Verifier.abi)
              const fragment = iface.getFunction('verifyExitProof')
              const calldata = iface.encodeFunctionData(fragment, payload)
              const tx = await provider.send('eth_call', [
                {
                  from: account,
                  to: '0x' + verifyContractAddress.substr(-40),
                  data: calldata,
                },
              ])

              if (tx) {
                toast.success(
                  (t) => (
                    <Stack>
                      <Typography sx={{ wordBreak: 'break-all' }}>{tx}</Typography>
                    </Stack>
                  ),
                  {
                    duration: 5000,
                  }
                )
              }
            } catch (e: any) {
              if (e?.code === -32603) {
              } else {
                toast.error(e?.message)
              }
              console.log(e)
            }
            setVerifyPending(false)
          }}
        >
          {verifyPending ? <CircularProgress sx={{ mr: 0.5 }} color="primary" size={14} /> : null}
          Verify
        </Button>
      ) : null}
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
            if (!provider || !contracts || !currentChain || !proofInfo?.proof_info || pending) {
              return
            }
            setPending(true)
            const payload = [
              {
                blockNumber: storedBlockInfo?.block_number,
                priorityOperations: storedBlockInfo?.priority_operations,
                pendingOnchainOperationsHash: storedBlockInfo?.pending_onchain_operations_hash,
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
              proofInfo.proof_info.amount,
              proofInfo.proof_info.proof?.proof,
            ]
            const iface = new Interface(MainContract.abi)
            const fragment = iface.getFunction('performExodus')
            const calldata = iface.encodeFunctionData(fragment, payload)
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
            if (e?.code === -32603) {
            } else {
              toast.error(e?.message)
            }
            console.log(e)
          }
          setPending(false)
        }}
      >
        {pending ? <CircularProgress sx={{ mr: 0.5 }} color="success" size={14} /> : null}
        Submit
      </Button>
    </Typography>
  ) : (
    <>
      {runningTaskId ? (
        mathjs.subtract(proofInfo.proof_info.id, runningTaskId) > 0 ? (
          <Typography sx={{ fontSize: 14 }} color="gray">
            Queue Position: {mathjs.subtract(proofInfo.proof_info.id, runningTaskId)}
          </Typography>
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
        )
      ) : null}
    </>
  )
})
