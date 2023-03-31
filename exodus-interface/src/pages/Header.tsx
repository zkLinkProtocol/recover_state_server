import { Box, Button, Stack, Typography } from '@mui/material'
import {
  useConnectWallet,
  useContracts,
  useCurrentChain,
  useSwitchNetwork,
} from '../store/home/hooks'
import { ConnectorNames } from '../connectors'
import { useWeb3React } from '@web3-react/core'
import { ChainInfo, chainList } from '../config/chains'
import { useEffect, useMemo, useState } from 'react'
import { styled } from '@mui/system'

const sxButton = {
  borderColor: 'rgba(33, 33, 33)',
  boxShadow: '2px 2px 0 rgba(11, 11, 11, 1)',
  '&:hover': {
    backgroundColor: 'rgb(142, 205, 30)',
  },
}

const Network = styled('div')({
  position: 'relative',
})
const NetworkOptions = styled(Box)({
  position: 'absolute',
  borderColor: 'rgba(33, 33, 33)',
  backgroundColor: '#FFFFFF',
  padding: '8px 0',
  top: '44px',
  boxShadow: '2px 2px 0 rgba(11, 11, 11, 1)',
})
const Dot = styled('div')({
  width: 6,
  height: 6,
  borderRadius: 3,
  marginLeft: 8,
  backgroundColor: 'rgb(142, 205, 30)',
})
const NetworkOption = styled(Stack)({
  whiteSpace: 'nowrap',
  padding: '8px 16px',
  cursor: 'pointer',
  flexDirection: 'row',
  alignItems: 'center',
  '&:hover': {
    backgroundColor: 'rgb(142, 205, 30)',

    '.dot': {
      background: 'rgb(122, 190, 50)',
    },
  },
})
export const encryptionAddress = (address?: string, start: number = 6, end: number = 4) => {
  if (!address) {
    return 'Unknown Address'
  }
  return `${address.substring(0, start)}...${address.substring(address.length - end)}`
}

export const Header = () => {
  const connectWallet = useConnectWallet()
  const { account, isActive } = useWeb3React()
  const currentChain = useCurrentChain()
  const contracts = useContracts()
  const [showOptions, setShowOptions] = useState(false)
  const switchNetwork = useSwitchNetwork()
  const chains: ChainInfo[] = useMemo(() => {
    if (!contracts) {
      return []
    }
    return Object.values(chainList).filter((v) => !!contracts[v.l2ChainId])
  }, [contracts])

  useEffect(() => {
    document.body.addEventListener('click', (e) => {
      setShowOptions(false)
    })
  })
  return (
    <Stack height="88px" spacing={1} alignItems="center" direction="row">
      <Typography variant="h6">ZKLINK</Typography>
      <Box flex="1"></Box>
      <Network>
        <Button
          sx={sxButton}
          variant="outlined"
          color={currentChain?.name ? 'inherit' : 'error'}
          onClick={(event) => {
            event.stopPropagation()
            setShowOptions(!showOptions)
          }}
        >
          {currentChain?.name ?? 'Known Network'}
        </Button>
        {chains?.length && showOptions ? (
          <NetworkOptions sx={{ border: 1 }}>
            {chains.map((v) => (
              <NetworkOption
                key={v.chainId}
                onClick={() => {
                  setShowOptions(false)
                  switchNetwork(v.chainId)
                }}
              >
                {v.name} {currentChain?.chainId === v.chainId ? <Dot className="dot" /> : null}
              </NetworkOption>
            ))}
          </NetworkOptions>
        ) : null}
      </Network>
      <Button
        sx={sxButton}
        variant="outlined"
        color="inherit"
        onClick={() => {
          connectWallet(ConnectorNames.Metamask)
        }}
      >
        {isActive ? encryptionAddress(account) : 'Connect Wallet'}
      </Button>
    </Stack>
  )
}
