import axios from 'axios'
import { API_HOST } from '../config'

export const http = axios.create({
  baseURL: API_HOST,
})
