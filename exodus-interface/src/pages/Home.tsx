import { Box, Container, Typography } from '@mui/material'
import { Header } from './Header'
import { L2Balances } from './L2Balances'
import { styled } from '@mui/system'

const Section = styled(Box)({
  backgroundColor: 'rgba(237, 237, 237)',
  padding: 16,
  marginBottom: 16,
  boxShadow: '4px 4px 0 rgb(218, 218, 218)',
})
export const Home = () => {
  return (
    <Container>
      <Header />
      <Section>
        <Box
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
        </Box>

        <L2Balances />
        <Box></Box>
      </Section>
    </Container>
  )
}
