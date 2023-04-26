import axios from 'axios'

export interface DunkirkResponse<T = any> {
  code: number
  data: T
  err_msg: null | string
}

export const http = axios.create({
  baseURL: '/api',
})
