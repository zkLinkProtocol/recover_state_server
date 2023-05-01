import { Box, Button, Container, TextField } from '@mui/material'
import { useState } from 'react'
import { useCurrentChain, useNetworks, useStoredBlockInfo } from '../store/home/hooks'
import { Header } from './Header'
import { useWeb3React } from '@web3-react/core'
import { toast } from 'react-hot-toast'
import { Interface, keccak256, AbiCoder } from 'ethers/lib/utils'
import MainContract from 'zklink-js-sdk/abi/ZkLink.json'
import { BigNumber } from 'ethers'

const abiCode = new AbiCoder()
export const VerifyStoredBlockHash = () => {
  const [txHash, setTxHash] = useState(
    '0x4904d6acf50dd3fdfb406825bd51dfc2abf7d85cdebb998f2d44f01427f03c07'
  )
  const { provider } = useWeb3React()
  const currentChain = useCurrentChain()
  const storedBlockInfo = useStoredBlockInfo(currentChain?.layerTwoChainId)
  const query = async () => {
    try {
      if (!provider) {
        throw new Error('Invalid provider')
      }
      if (!storedBlockInfo) {
        throw new Error('Invalid layer2 stored block info')
      }
      const r = await provider.getTransaction(txHash)

      const iface = new Interface(MainContract.abi)
      const fragment = iface.getFunction('executeBlocks')
      const calldata = iface.decodeFunctionData(fragment, r.data)
      const data = calldata[0][calldata[0].length - 1]
      const layerOneHash = keccak256(
        abiCode.encode(
          ['uint32', 'uint64', 'bytes32', 'uint256', 'bytes32', 'bytes32', 'bytes32'],
          data.storedBlock
        )
      )
      const layerTwoStoredBlockInfo = [
        storedBlockInfo.block_number,
        BigNumber.from(storedBlockInfo.priority_operations),
        storedBlockInfo.pending_onchain_operations_hash,
        BigNumber.from(storedBlockInfo.timestamp),
        storedBlockInfo.state_hash,
        storedBlockInfo.commitment,
        storedBlockInfo.sync_hash,
      ]
      const layerTwoHash = keccak256(
        abiCode.encode(
          ['uint32', 'uint64', 'bytes32', 'uint256', 'bytes32', 'bytes32', 'bytes32'],
          layerTwoStoredBlockInfo
        )
      )

      console.log(layerOneHash)
      console.log(layerTwoHash)
    } catch (e: any) {
      toast.error(e?.message)
      console.log(e)
    }
  }

  return (
    <Container>
      <Header></Header>
      <Box sx={{ p: 4 }}>
        <TextField
          id="outlined-basic"
          label="Outlined"
          variant="outlined"
          value={txHash}
          onChange={(event) => {
            setTxHash(event.currentTarget.value)
          }}
          size="small"
          fullWidth={true}
        />
        <Button variant="text" onClick={query}>
          Submit
        </Button>
      </Box>
    </Container>
  )
}
