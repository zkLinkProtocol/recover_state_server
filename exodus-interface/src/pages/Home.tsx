import { Container, Typography } from '@mui/material'
import { Header } from './Header'
import { L2Balances } from './L2Balances'
import { RouterProvider } from 'react-router'
import { createBrowserRouter } from 'react-router-dom'
import { History } from './History'

const router = createBrowserRouter([
  {
    path: '/',
    element: <L2Balances />,
  },
  {
    path: 'history',
    element: <History />,
  },
])

export const Home = () => {
  return (
    <Container>
      <RouterProvider router={router} />
    </Container>
  )
}
