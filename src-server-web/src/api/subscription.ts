import client from './client'

export interface Subscription {
  tier: string
  status: string
  expires_at: string | null
}

export async function getMySubscription(): Promise<Subscription> {
  const res = await client.get('/subscription/me')
  return res.data
}
