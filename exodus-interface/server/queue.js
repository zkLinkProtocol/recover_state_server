const { Client } = require('pg');
const { CronJob } = require('cron')
const connectionString = process.env.DATABASE_URL

let queueCount = 0

async function queryQueueCount() {
  const client = new Client({ connectionString })
  try {
    await client.connect()
    const result = await client.query('select count(*) from exit_proofs where created_at is null and finished_at is null;')
    const rowCount = parseInt(result.rows[0].count, 10)
    queueCount = rowCount
  } catch (error) {
    console.log(error)
  } finally {
    await client.end()
  }
}

const cron = new CronJob('* * * * *', async function () {
  await queryQueueCount()
})

async function initQueueCount() {
  cron.fireOnTick()
  cron.start()
}

function getQueueCount() {
  return queueCount
}

module.exports = {
  initQueueCount,
  getQueueCount
}