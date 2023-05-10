const dotenv = require('dotenv')
const path = require('path')

dotenv.config({ path: path.resolve(process.cwd(), '../.env') })

const contractAddresses = {}

async function initContracts() {
  const chainIds = process.env.CHAIN_IDS.split(',')

  chainIds.forEach(chainId => {
    const contractAddress = process.env[`CHAIN_${chainId}_CONTRACT_ADDRESS`]
    contractAddresses[chainId] = contractAddress
  })
}

async function getContracts() {
  return contractAddresses
}

module.exports = {
  initContracts,
  getContracts
}
