function boolean(envValue: string | boolean) {
  return envValue === 'true' || envValue === true ? true : false
}

export const ENV: 'devnet' | 'testnet' = process.env.REACT_APP_ENV! as 'devnet' | 'testnet'

export const API_HOST = process.env.REACT_APP_API_HOST
export const GITHUB_STATIC_PATH = process.env.REACT_APP_GITHUB_STATIC_HOST
