const { Client } = require('pg');
const { CronJob } = require('cron')
const connectionString = process.env.DATABASE_URL

let blocks

async function queryRecoverBlocks() {
  const client = new Client({ connectionString })
  try {
    await client.connect()
    const result = await client.query('SELECT COUNT(*) FROM blocks;')
    const rowCount = parseInt(result.rows[0].count - 1, 10)
    blocks = rowCount
  } catch (error) {
    console.log(error)
  } finally {
    await client.end()
  }
}

const cron = new CronJob('*/10 * * * * *', async function () {
  await queryRecoverBlocks()
})

async function initRecoverBlocks() {
  cron.fireOnTick()
  cron.start()
}

function getRecoverBlocks() {
  return blocks
}

module.exports = {
  initRecoverBlocks,
  getRecoverBlocks
}