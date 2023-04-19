import MockAdapter from 'axios-mock-adapter'
import { http } from '../api'

const mock = new MockAdapter(http, {
  onNoMatch: 'passthrough',
})

mock.onGet('/contracts').reply(200, {
  code: 0,
  data: {
    '2': '0x331a96b91f35051706680d96251931e26f4ba58a',
    '1': '0x517aa9dec0e297b744ac7ac8ddd8b127c1993055',
  },
  err_msg: null,
})
mock.onGet('/recover_progress').reply(200, {
  code: 0,
  data: {
    current_block: 1,
    total_verified_block: 1,
  },
  err_msg: null,
})
mock.onGet('/running_max_task_id').reply(200, {
  code: 0,
  data: {
    id: 100,
  },
  err_msg: null,
})
mock.onGet('/tokens').reply(200, {
  code: 0,
  data: {
    '58': {
      token_id: 58,
      symbol: 'FLOKI',
      addresses: { '2': '0x2d100d436b4a10f62dcdebe83be55321a4c19322' },
    },
    '49': {
      token_id: 49,
      symbol: 'BTC3S',
      addresses: { '2': '0xc7515e5df9d1fa3aed6754f3f58738918df7ad5c' },
    },
    '53': {
      token_id: 53,
      symbol: 'BTC5S',
      addresses: { '2': '0x964419c7a8fd86b5c787deeb3e0e643e7f400521' },
    },
    '57': {
      token_id: 57,
      symbol: 'GRAIL',
      addresses: { '2': '0xfc8112441ebc7fc3980a7dd039ab871e9018b092' },
    },
    '55': {
      token_id: 55,
      symbol: 'ETH5S',
      addresses: { '2': '0xfaac68110fa59cd950957be2afc702e942144aac' },
    },
    '48': {
      token_id: 48,
      symbol: 'BTC3L',
      addresses: { '2': '0x405e67861d084fe73a80c721595a2c2234f3f8cf' },
    },
    '17': {
      token_id: 17,
      symbol: 'USDT',
      addresses: {
        '1': '0x91e5d0c39e3f2de1d8cbbecca3604f6704fb3494',
        '2': '0x2b1d07f867b220fcc818e9d7ff4fcb08e63b2ae5',
      },
    },
    '43': {
      token_id: 43,
      symbol: 'wAVAX',
      addresses: { '2': '0x2796baed33862664c08b8ee5fa2d1283c79593b1' },
    },
    '1': { token_id: 1, symbol: 'USD', addresses: {} },
    '51': {
      token_id: 51,
      symbol: 'ETH3S',
      addresses: { '2': '0x7d1e13a4885acc18ae907cc6bd88d74aeb68bb63' },
    },
    '41': {
      token_id: 41,
      symbol: 'wETH',
      addresses: {
        '1': '0x329b54e5e5d0467cf7b04553f2e3aabb22372b4e',
        '2': '0xdfcf5b34da20f8e49b4e0517f4df4c8ab0fe94ad',
      },
    },
    '52': {
      token_id: 52,
      symbol: 'BTC5L',
      addresses: { '2': '0x296c45a98347955b2a38655ec364b641dac0e10d' },
    },
    '42': {
      token_id: 42,
      symbol: 'wMATIC',
      addresses: { '1': '0x76c9ef75f019496376c04dd19c38637cacce9e42' },
    },
    '19': {
      token_id: 19,
      symbol: 'BUSD',
      addresses: {
        '1': '0xb3ee6008fea338aef0ab4f775b044b279be65d7a',
        '2': '0xb5e8fbdcde0251851af328065c7e3dba6ab037cd',
      },
    },
    '56': {
      token_id: 56,
      symbol: 'FOX',
      addresses: { '2': '0xaa8d94e21537c3cbd8cea329803da3996ba353d4' },
    },
    '50': {
      token_id: 50,
      symbol: 'ETH3L',
      addresses: { '2': '0x22343f93f70af0c88b25223111bcd35b9c8400dd' },
    },
    '18': {
      token_id: 18,
      symbol: 'USDC',
      addresses: {
        '2': '0x2645b73c58702ab81904a6cabdf63340b4ce29d3',
        '1': '0xa581b8e2b305d3a8a1ef2442159e4d46bc9fcc50',
      },
    },
    '40': {
      token_id: 40,
      symbol: 'wBTC',
      addresses: {
        '1': '0x0fc3283a6d40550185a4d8cbd00030194475bbc4',
        '2': '0xc6df93a49198bb902abc0231955ec77ae0cc34aa',
      },
    },
    '47': {
      token_id: 47,
      symbol: 'QUICK',
      addresses: { '1': '0x01230636333b7260f466948639645af865b79b68' },
    },
    '54': {
      token_id: 54,
      symbol: 'ETH5L',
      addresses: { '2': '0x50b66cb3e58d988ee7feb7b8d62081d9782ef3b3' },
    },
    '45': {
      token_id: 45,
      symbol: 'AUTO',
      addresses: {
        '2': '0x8f8b0bfb8458f73249024f22b4cf7b6c0eb60996',
        '1': '0x1f34934e3165b3e5428f6a4d873d2620302c7223',
      },
    },
    '46': {
      token_id: 46,
      symbol: 'JOE',
      addresses: { '2': '0x145407c16831512459f6d4a672e16b4dcfb3fc40' },
    },
    '34': {
      token_id: 34,
      symbol: 'AVAX',
      addresses: { '2': '0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee' },
    },
    '33': {
      token_id: 33,
      symbol: 'MATIC',
      addresses: { '1': '0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee' },
    },
    '44': {
      token_id: 44,
      symbol: 'SYN',
      addresses: {
        '1': '0x753130a418eccfa6b9e601c6b6d418871087e9b1',
        '2': '0x2cf622797cae91054911ab31c6d0aa480b6d82bc',
      },
    },
  },
  err_msg: null,
})
mock.onPost('/get_stored_block_info').reply(200, {
  code: 0,
  data: {
    block_number: 2260,
    priority_operations: 0,
    pending_onchain_operations_hash:
      '0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470',
    timestamp: '0x64199e2d',
    state_hash: '0x00142ea9acc5e6d85e8865a43d3ce45279608d1488b7f308120ffb28d8cc9c47',
    commitment: '0x13952e7370d7cb4df472f4a7788909514f6bf5978d97ef07ae711d6a00a21b4b',
    sync_hash: '0x1068d5e8a812de313dca7fdd575539ef6c632620c9cc1ad01f7275fc9ee907c5',
  },
  err_msg: null,
})
mock.onPost(/^\/get_balances/).reply(200, {
  code: 0,
  data: { '1': { '1': '9826567799000000000000', '41': '99900000000000000' } },
  err_msg: null,
})
mock.onPost('/get_proofs_by_page').reply((config) => {
  return [
    200,
    {
      code: 0,
      data: {
        total_completed_num: 1234,
        proofs: [
          {
            exit_info: {
              chain_id: 2,
              account_address: '0x0000000000000000000000000000000000000000000000000000000000000000',
              account_id: 0,
              sub_account_id: 0,
              l1_target_token: 50,
              l2_source_token: 50,
            },
            proof_info: {
              id: 1,
              amount: null,
              proof: null,
            },
          },
        ],
      },
      err_msg: null,
    },
  ]
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
            proof_info: {
              id: 234,
              amount: '123456',
              proof: '0x4566521312321321321321',
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
            proof_info: {
              id: 4103,
              amount: '123456',
              proof: '0x4566521312321321321321',
            },
          },
        ],
        err_msg: '',
      },
    ]
  }
  if (data.token_id === 1) {
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
            proof_info: {
              id: 1000,
              amount: null,
              proof: null,
            },
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
            proof_info: {
              id: 1001,
              amount: null,
              proof: null,
            },
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
            proof_info: {
              id: 1002,
              amount: null,
              proof: null,
            },
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
            proof_info: {
              id: 1003,
              amount: null,
              proof: null,
            },
          },
        ],
        err_msg: '',
      },
    ]
  }

  return [
    200,
    {
      code: 0,
      data: null,
      err_msg: '',
    },
  ]
})
mock.onPost('/generate_proof_tasks_by_token').reply(200, {
  code: 0,
  data: null,
  err_msg: '',
})
