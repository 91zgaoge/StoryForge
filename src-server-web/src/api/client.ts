import axios from 'axios'

const client = axios.create({ baseURL: import.meta.env.VITE_API_URL || '/api' })

client.interceptors.request.use(config => {
  const token = localStorage.getItem('sf_token')
  if (token) config.headers.Authorization = `Bearer ${token}`
  return config
})

client.interceptors.response.use(
  resp => resp,
  err => {
    if (err.response?.status === 401) {
      localStorage.removeItem('sf_token')
      localStorage.removeItem('sf_role')
      window.location.href = '/login'
    }
    return Promise.reject(err)
  }
)

export default client
