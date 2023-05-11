import { Box, Button, Stack, Typography } from '@mui/material'
import {
  useConnectWallet,
  useContracts,
  useCurrentChain,
  useNetworks,
  useSwitchNetwork,
} from '../store/home/hooks'
import { ConnectorNames } from '../connectors'
import { useWeb3React } from '@web3-react/core'
import { useEffect, useMemo, useState } from 'react'
import { styled } from '@mui/system'
import logoUrl from './../assets/zklink-logo.png'
import { Link, useLocation } from 'react-router-dom'
import { NetworkInfo } from '../store/home/types'
import { useEffectOnce } from 'usehooks-ts'
import axios from 'axios'
import { NODE_NAME } from '../config'

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
const NetworkOptions = styled(Box)(({ theme }) => ({
  position: 'absolute',
  borderColor: 'rgba(33, 33, 33)',
  backgroundColor: '#FFFFFF',
  padding: '8px 0',
  top: '44px',
  boxShadow: '2px 2px 0 rgba(11, 11, 11, 1)',
  zIndex: 10,

  [theme.breakpoints.down('md')]: {
    top: 'auto',
    bottom: '44px',
  },
}))
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
const Nav = styled(Stack)({
  flex: 1,
  flexDirection: 'row',
  padding: '0 40px',

  a: {
    color: '#333',
    textDecoration: 'none',
    fontSize: 20,
    margin: '0 20px',
    transition: 'color .2s ease',

    '&:hover, &.active': {
      color: '#2e7d32',
    },
    '&.active': {
      color: '#2e7d32',
    },
  },
})
const Account = styled(Stack)(({ theme }) => ({
  flexDirection: 'row',
  justifyContent: 'flex-end',
  padding: 16,
  [theme.breakpoints.down('md')]: {
    backgroundColor: '#FFF',
    position: 'fixed',
    bottom: 0,
    right: 0,
    left: 0,
    zIndex: 20,
  },
}))
export const encryptionAddress = (address?: string, start: number = 6, end: number = 4) => {
  if (!address) {
    return 'Unknown Address'
  }
  return `${address.substring(0, start)}...${address.substring(address.length - end)}`
}

export const Header = () => {
  const location = useLocation()
  const networks = useNetworks()
  const connectWallet = useConnectWallet()
  const { account, isActive } = useWeb3React()
  const currentChain = useCurrentChain()
  const contracts = useContracts()
  const [showOptions, setShowOptions] = useState(false)
  const switchNetwork = useSwitchNetwork()
  const [showNode, setShowNode] = useState<boolean>()
  const chains: NetworkInfo[] = useMemo(() => {
    if (!contracts) {
      return []
    }
    return networks.filter((v) => !!contracts[v.layerTwoChainId])
  }, [contracts, networks])

  useEffectOnce(() => {
    document.body.addEventListener('click', (e) => {
      setShowOptions(false)
    })
  })

  return (
    <Stack height="88px" spacing={1} alignItems="center" direction="row">
      <img src={logoUrl} width="26" />
      <Typography variant="h5">zkLink</Typography>

      <Stack flexDirection="row" alignItems="center">
        {showNode || (NODE_NAME != undefined && NODE_NAME != '') ? (
          <Typography sx={{ marginRight: 1 }} variant="h5">
            /
          </Typography>
        ) : null}
        <img
          style={{ display: 'none', marginRight: 8 }}
          src={`${process.env.PUBLIC_URL}/node.png`}
          onLoad={function (event) {
            event.currentTarget.style.display = 'block'
            setShowNode(true)
          }}
          height="26"
        />
        <img
          style={{ display: 'none', marginRight: 8 }}
          src={`${process.env.PUBLIC_URL}/node.svg`}
          onLoad={function (event) {
            setShowNode(true)
            event.currentTarget.style.display = 'block'
          }}
          height="26"
        />
        {NODE_NAME ? (
          <Typography sx={{ marginRight: 1 }} variant="h5">
            {NODE_NAME}
          </Typography>
        ) : null}
      </Stack>

      <Nav>
        <Link className={location.pathname === '/' ? 'active' : ''} to={'/'}>
          Home
        </Link>
        <Link className={location.pathname === '/history' ? 'active' : ''} to={'/history'}>
          History
        </Link>
      </Nav>
      <Account>
        <Network sx={{ mr: 2 }}>
          <Button
            sx={sxButton}
            variant="outlined"
            color={currentChain?.name ? 'inherit' : 'error'}
            onClick={(event) => {
              event.stopPropagation()
              setShowOptions(!showOptions)
            }}
          >
            {currentChain?.name ?? 'Unknown Network'}
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
      </Account>
    </Stack>
  )
}
