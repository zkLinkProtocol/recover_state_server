import { FC } from 'react'
import { styled } from '@mui/system'
import { STATIC_HOST } from '../config'

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
        verticalAlign: 'middle',
      }}
      src={`${STATIC_HOST}/token/icons/default/${symbol?.toLowerCase()}.svg`}
    />
  )
}
