import { styled, Container, Typography } from '@mui/material'
import { Header } from './Header'
import { L2Balances } from './L2Balances'
import { RouterProvider } from 'react-router'
import { createBrowserRouter } from 'react-router-dom'
import { History } from './History'
import { SectionPendingBalance } from './PendingBalance'

const router = createBrowserRouter([
  {
    path: '/',
    element: (
      <>
        <Header />

        <SectionPendingBalance />
        <L2Balances />
      </>
    ),
  },
  {
    path: 'history',
    element: <History />,
  },
])

const HomeContainer = styled(Container)(({ theme }) => ({
  [theme.breakpoints.down('md')]: {
    paddingBottom: '80px',
  },
}))

export const Home = () => {
  return (
    <HomeContainer>
      <RouterProvider router={router} />
    </HomeContainer>
  )
}
