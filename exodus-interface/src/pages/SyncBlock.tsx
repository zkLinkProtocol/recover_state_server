import { Typography, Stack } from '@mui/material'
import { styled } from '@mui/system'
import LinearProgress, { linearProgressClasses } from '@mui/material/LinearProgress'

const BorderLinearProgress = styled(LinearProgress)(({ theme }) => ({
  height: 10,
  borderRadius: 5,
  [`&.${linearProgressClasses.colorPrimary}`]: {
    backgroundColor: theme.palette.grey[400],
  },
  [`& .${linearProgressClasses.bar}`]: {
    borderRadius: 5,
    backgroundColor: theme.palette.success.main,
  },
}))

export const SyncBlock = () => {
  return (
    <Stack
      sx={{
        textAlign: 'center',
        width: '80%',
        m: '160px auto',
      }}
      spacing={2}
    >
      <BorderLinearProgress variant="determinate" value={50} />
      <Typography sx={{ fontSize: 18 }}>
        Waiting for the block to finish syncing. (100 / 1000)
      </Typography>
    </Stack>
  )
}
