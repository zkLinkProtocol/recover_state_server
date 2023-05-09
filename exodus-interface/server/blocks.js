const { Client } = require('pg');

const connectionString = 'postgres://postgres:postgres@localhost/plasma'

async function getBlocksRowCount() {
  const client = new Client({ connectionString });
  try {
    await client.connect();
    const result = await client.query('SELECT COUNT(*) FROM blocks;');
    const rowCount = parseInt(result.rows[0].count - 1, 10);
    console.log('Total rows in blocks table:', rowCount);
    return rowCount;
  } catch (error) {
    console.error('Error while fetching row count:', error);
    return Promise.reject(error)
  } finally {
    await client.end();
  }
}

module.exports = {
  getBlocksRowCount
}