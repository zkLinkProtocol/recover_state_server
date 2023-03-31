import { FC } from 'react'
import { styled } from '@mui/system'

const Icon = styled('img')({
  overflow: 'hidden',
  borderRadius: '50%',
})

export const TokenIcon: FC<{ symbol?: string; size?: number }> = ({ symbol, size = 24 }) => {
  if (!symbol) {
    return null
  }
  return (
    <Icon
      sx={{
        width: size,
        height: size,
      }}
      src={`https://static.zk.link/token/icons/default/${symbol?.toLowerCase()}.svg`}
    />
  )
}
