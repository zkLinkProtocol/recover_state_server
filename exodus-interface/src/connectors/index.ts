import { Web3ReactHooks } from '@web3-react/core'
import { MetaMask } from '@web3-react/metamask'
import { hooks as metamaskHooks, metamask } from './metamask'

export enum ConnectorNames {
  Metamask = 'MetaMask',
}

export function connectorByName(connectorName: ConnectorNames) {
  switch (connectorName) {
    case ConnectorNames.Metamask:
      return metamask
  }
}

export const connectors: [MetaMask, Web3ReactHooks][] = [[metamask, metamaskHooks]]
