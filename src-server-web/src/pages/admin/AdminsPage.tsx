import { useEffect, useState } from 'react'
import axios from 'axios'
import { Shield, UserPlus } from 'lucide-react'
import { listUsers, setUserRole, type AdminUser } from '../../api/admin'

const API_BASE = import.meta.env.VITE_API_URL || '/api'

interface MeInfo {
  id?: string
  email?: string
}

function formatDate(iso: string): string {
  return new Date(iso).toISOString().slice(0, 10)
}

export default function AdminsPage() {
  const [admins, setAdmins] = useState<AdminUser[]>([])
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState('')
  const [me, setMe] = useState<MeInfo | null>(null)

  const [email, setEmail] = useState('')
  const [isPromoting, setIsPromoting] = useState(false)
  const [busyId, setBusyId] = useState<string | null>(null)
  const [toast, setToast] = useState('')

  const refresh = () => {
    listUsers()
      .then(users => {
        setAdmins(users.filter(u => u.role === 'admin'))
        setError('')
      })
      .catch(() => setError('加载管理员失败'))
      .finally(() => setIsLoading(false))
  }

  useEffect(() => {
    refresh()
    // 与 DashboardPage 一致：用 sf_token 调 /auth/me 拿当前用户，标记「我」的行
    const token = localStorage.getItem('sf_token')
    if (!token) return
    axios
      .get(`${API_BASE}/auth/me`, {
        headers: { Authorization: `Bearer ${token}` },
      })
      .then(res => setMe(res.data))
      .catch(() => {})
  }, [])

  const isMe = (u: AdminUser): boolean =>
    !!me && (u.id === me.id || (!!u.email && u.email === me.email))

  const handlePromote = async () => {
    const q = email.trim()
    if (!q) return
    setIsPromoting(true)
    setToast('')
    try {
      const matches = await listUsers(q)
      const target = matches.find(u => u.email === q)
      if (!target) {
        setToast('未找到邮箱精确匹配的用户')
        return
      }
      if (!window.confirm(`确认将 ${target.email} 提拔为管理员？`)) return
      await setUserRole(target.id, 'admin')
      setToast(`已将 ${target.email} 提拔为管理员`)
      setEmail('')
      refresh()
    } catch {
      setToast('提拔失败，请重试')
    } finally {
      setIsPromoting(false)
    }
  }

  const handleDemote = async (u: AdminUser) => {
    if (!window.confirm(`确认将 ${u.email} 降级为普通用户？`)) return
    setBusyId(u.id)
    setToast('')
    try {
      await setUserRole(u.id, 'user')
      setToast(`已将 ${u.email} 降级为普通用户`)
      refresh()
    } catch {
      setToast('降级失败，请重试')
    } finally {
      setBusyId(null)
    }
  }

  return (
    <div className="space-y-6">
      {/* 提拔管理员 */}
      <div className="bg-cinema-900/50 border border-cinema-800/50 rounded-2xl p-6">
        <h2 className="text-lg font-semibold text-white flex items-center gap-2">
          <UserPlus className="w-5 h-5 text-cinema-gold" />
          提拔管理员
        </h2>
        <div className="mt-4 flex items-center gap-3">
          <input
            aria-label="用户邮箱"
            type="text"
            value={email}
            onChange={e => setEmail(e.target.value)}
            placeholder="输入用户邮箱"
            className="px-3 py-2 w-72 bg-cinema-800/50 border border-cinema-700/50 rounded-lg text-sm text-white focus:outline-none focus:border-cinema-gold/50"
          />
          <button
            onClick={handlePromote}
            disabled={isPromoting || !email.trim()}
            className="px-4 py-2 bg-cinema-gold text-cinema-900 rounded-lg text-sm font-medium hover:bg-cinema-gold-light transition-colors disabled:opacity-50"
          >
            提拔为管理员
          </button>
          {toast && <span className="text-sm text-gray-400">{toast}</span>}
        </div>
      </div>

      {/* 管理员列表 */}
      <div className="bg-cinema-900/50 border border-cinema-800/50 rounded-2xl p-6">
        <h2 className="text-lg font-semibold text-white flex items-center gap-2">
          <Shield className="w-5 h-5 text-cinema-gold" />
          管理员列表
        </h2>
        {isLoading ? (
          <p className="text-sm text-gray-400 mt-4">加载中...</p>
        ) : error ? (
          <p className="text-sm text-red-400 mt-4">{error}</p>
        ) : admins.length === 0 ? (
          <p className="text-sm text-gray-400 mt-4">暂无管理员</p>
        ) : (
          <div className="mt-4 overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="text-left text-xs text-gray-500 border-b border-cinema-800/50">
                  <th className="pb-2 pr-4">邮箱</th>
                  <th className="pb-2 pr-4">昵称</th>
                  <th className="pb-2 pr-4">Tier</th>
                  <th className="pb-2 pr-4">注册时间</th>
                  <th className="pb-2">操作</th>
                </tr>
              </thead>
              <tbody>
                {admins.map(u => (
                  <tr key={u.id} className="border-b border-cinema-800/30">
                    <td className="py-3 pr-4 text-white">
                      {u.email || '—'}
                      {isMe(u) && (
                        <span className="ml-2 px-1.5 py-0.5 text-xs rounded bg-cinema-gold/20 text-cinema-gold">
                          我
                        </span>
                      )}
                    </td>
                    <td className="py-3 pr-4 text-gray-300">{u.display_name || '—'}</td>
                    <td className="py-3 pr-4 text-gray-300">
                      {u.tier === 'pro' ? 'Pro' : '免费'}
                    </td>
                    <td className="py-3 pr-4 text-gray-400">{formatDate(u.created_at)}</td>
                    <td className="py-3">
                      {!isMe(u) && (
                        <button
                          onClick={() => handleDemote(u)}
                          disabled={busyId === u.id}
                          className="text-red-400 hover:text-red-300 text-sm transition-colors disabled:opacity-50"
                        >
                          降级
                        </button>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  )
}
