import axios from 'axios'
import { API_HOST } from '../config'

export interface DunkirkResponse<T = any> {
  code: number
  data: T
  err_msg: null | string
}

export const http = axios.create({
  baseURL: API_HOST,
})
