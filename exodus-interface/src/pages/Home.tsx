import { styled, Container, Typography } from '@mui/material'
import { Header } from './Header'
import { SectionL2Balance } from './L2Balance'
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
        <SectionL2Balance />
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
