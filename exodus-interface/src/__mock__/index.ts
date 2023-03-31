import axios from 'axios'
import MockAdapter from 'axios-mock-adapter'
import { http } from '../api'

// This sets the mock adapter on the default instance
const mock = new MockAdapter(http)

// Mock any GET request to /users
// arguments for reply are (status, data, headers)
mock.onGet('/contracts').reply(200, {
  code: 0,
  data: {
    '2': '0x2a038a0549d63fa95a87a84662c9e24358b4f81f',
    '1': '0xb404593d990d8afccf40fd9c5a0f906e1e1c77a1',
  },
  err_msg: '',
})

mock.onGet('/tokens').reply(200, {
  code: 0,
  data: {
    '1': {
      token_id: 1,
      addresses: {},
    },
    '17': {
      token_id: 17,
      addresses: {
        '2': '0x2b1d07f867b220fcc818e9d7ff4fcb08e63b2ae5',
        '1': '0x91e5d0c39e3f2de1d8cbbecca3604f6704fb3494',
      },
    },
    '41': {
      token_id: 41,
      addresses: {
        '2': '0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee',
      },
    },
    '40': {
      token_id: 40,
      addresses: {
        '1': '0x0fc3283a6d40550185a4d8cbd00030194475bbc4',
        '2': '0xc6df93a49198bb902abc0231955ec77ae0cc34aa',
      },
    },
  },
  err_msg: '',
})
mock.onPost('/get_stored_block_info').reply(200, {
  code: 0,
  data: {
    block_number: 1350,
    priority_operations: 0,
    pending_onchain_operations_hash:
      '0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470',
    timestamp: '0x6401bdbb',
    state_hash: '0x00dd8c06f1ce5361723d9727d44320a0f742a162c71a39397ca6913ff98d0b21',
    commitment: '0xb8c806408e65e9e28fe85b697bb21e22ebe4d494c043ed19acf0bf23baeab7a8',
    sync_hash: '0xa5bdf341496b3a092d4826d7df7979f10acc59989d76c6d648c0e52eb0506ea7',
  },
  err_msg: '',
})
mock.onPost(/^\/get_balances/).reply(200, {
  code: 0,
  data: {
    '0': {
      '1': '1498994167999999999973',
      '17': '1498994167999999999973',
    },
    '1': {
      '40': '23498994167999999999973',
      '41': '1498994167999999999973',
    },
  },
  err_msg: '',
})
mock.onPost('/get_proofs_by_token').reply((config) => {
  const data = JSON.parse(config.data)
  if (data.token_id === 41) {
    return [
      200,
      {
        code: 0,
        data: [
          {
            exit_info: {
              chain_id: 1,
              account_address: '0x04EBC47B5B0FA6E283DDC3C3B21DC9CD6B036D38',
              account_id: 12,
              sub_account_id: 1,
              l1_target_token: 41,
              l2_source_token: 41,
            },
            amount: '23412818237457823',
            proof: {
              inputs: [],
              proof: [],
            },
          },
          {
            exit_info: {
              chain_id: 2,
              account_address: '0x04EBC47B5B0FA6E283DDC3C3B21DC9CD6B036D38',
              account_id: 12,
              sub_account_id: 1,
              l1_target_token: 41,
              l2_source_token: 41,
            },
            amount: '23412818237457823',
            proof: {
              inputs: [],
              proof: [],
            },
          },
        ],
        err_msg: '',
      },
    ]
  }
  if (data.token_id !== 1) {
    return [
      200,
      {
        code: 0,
        data: null,
        err_msg: '',
      },
    ]
  }
  return [
    200,
    {
      code: 0,
      data: [
        {
          exit_info: {
            chain_id: 1,
            account_address: '0x04EBC47B5B0FA6E283DDC3C3B21DC9CD6B036D38',
            account_id: 12,
            sub_account_id: 1,
            l1_target_token: 17,
            l2_source_token: 1,
          },
          amount: null,
          proof: null,
        },
        {
          exit_info: {
            chain_id: 1,
            account_address: '0x04EBC47B5B0FA6E283DDC3C3B21DC9CD6B036D38',
            account_id: 12,
            sub_account_id: 1,
            l1_target_token: 18,
            l2_source_token: 1,
          },
          amount: null,
          proof: null,
        },
        {
          exit_info: {
            chain_id: 2,
            account_address: '0x04EBC47B5B0FA6E283DDC3C3B21DC9CD6B036D38',
            account_id: 12,
            sub_account_id: 1,
            l1_target_token: 17,
            l2_source_token: 1,
          },
          amount: null,
          proof: null,
        },
        {
          exit_info: {
            chain_id: 2,
            account_address: '0x04EBC47B5B0FA6E283DDC3C3B21DC9CD6B036D38',
            account_id: 12,
            sub_account_id: 1,
            l1_target_token: 18,
            l2_source_token: 1,
          },
          amount: null,
          proof: null,
        },
      ],
      err_msg: '',
    },
  ]
})
mock.onPost('/generate_proof_tasks_by_token').reply(200, {
  code: 0,
  data: null,
  err_msg: '',
})

mock.onAny().passThrough()
