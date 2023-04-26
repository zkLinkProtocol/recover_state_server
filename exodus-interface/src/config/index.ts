function boolean(envValue: string | boolean) {
  return envValue === 'true' || envValue === true ? true : false
}

export const ENV: 'devnet' | 'testnet' = process.env.REACT_APP_ENV! as 'devnet' | 'testnet'

export const STATIC_HOST = process.env.REACT_APP_STATIC_HOST

export const RUNNING_TASK_ID_DELAY = 10000
export const RECOVER_PROGRESS_DELAY = 10000
export const PENDING_BALANCE_DELAY = 10000
