import { Typography, Stack } from '@mui/material'
import { styled } from '@mui/system'
import LinearProgress, { linearProgressClasses } from '@mui/material/LinearProgress'
import { useRecoverProgress } from '../store/home/hooks'
import * as mathjs from 'mathjs'

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
  const recoverProgress = useRecoverProgress()
  if (!recoverProgress) {
    return <Typography>Loading...</Typography>
  }
  return (
    <Stack
      sx={{
        textAlign: 'center',
        width: '80%',
        m: '160px auto',
      }}
      spacing={2}
    >
      <BorderLinearProgress
        variant="determinate"
        value={mathjs.multiply(
          mathjs.divide(recoverProgress!.current_block, recoverProgress!.total_verified_block),
          100
        )}
      />
      <Typography sx={{ fontSize: 18 }}>
        {recoverProgress?.current_block} / {recoverProgress?.total_verified_block}
      </Typography>
      <Typography sx={{ fontSize: 18 }} color="gray">
        Waiting for the block to finish syncing.
      </Typography>
    </Stack>
  )
}
