import { ThemeProvider } from '@mui/material'
import { theme } from './theme'
import { Web3ReactProvider } from '@web3-react/core'
import { connectors } from './connectors'
import { Home } from './pages/Home'
import { Updater } from './store/Updater'
import { Provider } from 'react-redux'
import { store } from './store'
import { Toaster } from 'react-hot-toast'

function App() {
  return (
    <Provider store={store}>
      <ThemeProvider theme={theme}>
        <Web3ReactProvider connectors={connectors}>
          <Home />
          <Updater />
          <Toaster />
        </Web3ReactProvider>
      </ThemeProvider>
    </Provider>
  )
}

export default App
