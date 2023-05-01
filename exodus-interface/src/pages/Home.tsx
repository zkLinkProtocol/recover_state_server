import { styled, Container, Typography } from '@mui/material'
import { Header } from './Header'
import { SectionL2Balance } from './L2Balance'
import { RouterProvider } from 'react-router'
import { createHashRouter } from 'react-router-dom'
import { History } from './History'
import { SectionPendingBalance } from './PendingBalance'
import { VerifyStoredBlockHash } from './VerifyStoredBlockHash'

const router = createHashRouter([
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
  {
    path: 'verify_stored_block_hash',
    element: <VerifyStoredBlockHash />,
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
