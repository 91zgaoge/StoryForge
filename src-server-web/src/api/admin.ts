import client from './client'

export interface AdminUser {
  id: string
  email: string | null
  display_name: string | null
  role: string
  tier: string | null
  expires_at: string | null
  disabled_at: string | null
  created_at: string
}

export interface InviteCode {
  code: string
  max_uses: number
  used_count: number
  grant_pro_days: number | null
  note: string | null
  created_at: string
  revoked_at: string | null
}

export interface CreateInviteCodesRequest {
  count: number
  max_uses: number
  grant_pro_days?: number
  note?: string
}

export async function listUsers(q?: string): Promise<AdminUser[]> {
  const res = await client.get('/admin/users', { params: q ? { q } : {} })
  return res.data
}

export async function setUserRole(id: string, role: string) {
  const res = await client.post(`/admin/users/${id}/role`, { role })
  return res.data
}

export async function disableUser(id: string) {
  const res = await client.post(`/admin/users/${id}/disable`)
  return res.data
}

export async function enableUser(id: string) {
  const res = await client.post(`/admin/users/${id}/enable`)
  return res.data
}

export async function setUserSubscription(id: string, tier: string, days?: number) {
  const res = await client.post(`/admin/users/${id}/subscription`, { tier, days })
  return res.data
}

export async function listInviteCodes(): Promise<InviteCode[]> {
  const res = await client.get('/admin/invite-codes')
  return res.data
}

export async function createInviteCodes(body: CreateInviteCodesRequest): Promise<{ codes: string[] }> {
  const res = await client.post('/admin/invite-codes', body)
  return res.data
}

export async function revokeInviteCode(code: string) {
  const res = await client.post(`/admin/invite-codes/${code}/revoke`)
  return res.data
}
