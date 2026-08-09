import { useEffect, useState } from 'react'
import { Users, Search } from 'lucide-react'
import {
  listUsers,
  disableUser,
  enableUser,
  setUserSubscription,
  type AdminUser,
} from '../../api/admin'

function formatDate(iso: string): string {
  return new Date(iso).toISOString().slice(0, 10)
}

function displayTier(tier: string | null): string {
  return tier === 'pro' ? 'Pro' : '免费'
}

function displayRole(role: string): string {
  return role === 'admin' ? '管理员' : '用户'
}

export default function UsersPage() {
  const [users, setUsers] = useState<AdminUser[]>([])
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState('')
  const [query, setQuery] = useState('')
  const [busyId, setBusyId] = useState<string | null>(null)
  const [toast, setToast] = useState('')

  const refresh = (q?: string) => {
    listUsers(q)
      .then(setUsers)
      .catch(() => setError('加载用户失败'))
      .finally(() => setIsLoading(false))
  }

  useEffect(() => {
    refresh()
  }, [])

  const handleSearch = (value: string) => {
    setQuery(value)
    refresh(value)
  }

  const runAction = async (
    user: AdminUser,
    action: () => Promise<unknown>,
    successMsg: string
  ) => {
    setBusyId(user.id)
    setToast('')
    try {
      await action()
      setToast(successMsg)
      refresh(query)
    } catch {
      setToast('操作失败，请重试')
    } finally {
      setBusyId(null)
    }
  }

  const handleGrant = (user: AdminUser, days: number) =>
    runAction(user, () => setUserSubscription(user.id, 'pro', days), `已为 ${user.email} 赠 Pro ${days} 天`)

  const handleFree = (user: AdminUser) =>
    runAction(user, () => setUserSubscription(user.id, 'free'), `已将 ${user.email} 改为免费`)

  const handleToggleDisabled = (user: AdminUser) => {
    if (!user.disabled_at && !window.confirm(`确认禁用 ${user.email}？`)) return
    runAction(
      user,
      () => (user.disabled_at ? enableUser(user.id) : disableUser(user.id)),
      user.disabled_at ? `已启用 ${user.email}` : `已禁用 ${user.email}`
    )
  }

  return (
    <div className="bg-cinema-900/50 border border-cinema-800/50 rounded-2xl p-6">
      <div className="flex items-center justify-between gap-4 flex-wrap">
        <h2 className="text-lg font-semibold text-white flex items-center gap-2">
          <Users className="w-5 h-5 text-cinema-gold" />
          用户管理
        </h2>
        <div className="relative">
          <Search className="w-4 h-4 text-gray-500 absolute left-3 top-1/2 -translate-y-1/2" />
          <input
            aria-label="搜索用户"
            type="text"
            value={query}
            onChange={e => handleSearch(e.target.value)}
            placeholder="搜索邮箱 / 昵称"
            className="pl-9 pr-3 py-2 w-64 bg-cinema-800/50 border border-cinema-700/50 rounded-lg text-sm text-white focus:outline-none focus:border-cinema-gold/50"
          />
        </div>
      </div>
      {toast && <p className="text-sm text-gray-400 mt-3">{toast}</p>}

      {isLoading ? (
        <p className="text-sm text-gray-400 mt-4">加载中...</p>
      ) : error ? (
        <p className="text-sm text-red-400 mt-4">{error}</p>
      ) : users.length === 0 ? (
        <p className="text-sm text-gray-400 mt-4">暂无用户</p>
      ) : (
        <div className="mt-4 overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="text-left text-xs text-gray-500 border-b border-cinema-800/50">
                <th className="pb-2 pr-4">邮箱</th>
                <th className="pb-2 pr-4">昵称</th>
                <th className="pb-2 pr-4">Tier</th>
                <th className="pb-2 pr-4">角色</th>
                <th className="pb-2 pr-4">状态</th>
                <th className="pb-2 pr-4">注册时间</th>
                <th className="pb-2">操作</th>
              </tr>
            </thead>
            <tbody>
              {users.map(u => (
                <tr key={u.id} className="border-b border-cinema-800/30">
                  <td className="py-3 pr-4 text-white">{u.email || '—'}</td>
                  <td className="py-3 pr-4 text-gray-300">{u.display_name || '—'}</td>
                  <td className="py-3 pr-4">
                    {u.tier === 'pro' ? (
                      <span className="text-cinema-gold">{displayTier(u.tier)}</span>
                    ) : (
                      <span className="text-gray-300">{displayTier(u.tier)}</span>
                    )}
                  </td>
                  <td className="py-3 pr-4 text-gray-300">{displayRole(u.role)}</td>
                  <td className="py-3 pr-4">
                    {u.disabled_at ? (
                      <span className="text-red-400">禁用</span>
                    ) : (
                      <span className="text-green-400">正常</span>
                    )}
                  </td>
                  <td className="py-3 pr-4 text-gray-400">{formatDate(u.created_at)}</td>
                  <td className="py-3">
                    <div className="flex items-center gap-3 flex-wrap">
                      <button
                        onClick={() => handleGrant(u, 30)}
                        disabled={busyId === u.id}
                        className="text-cinema-gold hover:text-cinema-gold-light text-sm transition-colors disabled:opacity-50"
                      >
                        赠 Pro 30 天
                      </button>
                      <button
                        onClick={() => handleGrant(u, 90)}
                        disabled={busyId === u.id}
                        className="text-cinema-gold hover:text-cinema-gold-light text-sm transition-colors disabled:opacity-50"
                      >
                        赠 Pro 90 天
                      </button>
                      {u.tier === 'pro' && (
                        <button
                          onClick={() => handleFree(u)}
                          disabled={busyId === u.id}
                          className="text-gray-400 hover:text-white text-sm transition-colors disabled:opacity-50"
                        >
                          改为免费
                        </button>
                      )}
                      {u.disabled_at ? (
                        <button
                          onClick={() => handleToggleDisabled(u)}
                          disabled={busyId === u.id}
                          className="text-green-400 hover:text-green-300 text-sm transition-colors disabled:opacity-50"
                        >
                          启用
                        </button>
                      ) : (
                        <button
                          onClick={() => handleToggleDisabled(u)}
                          disabled={busyId === u.id}
                          className="text-red-400 hover:text-red-300 text-sm transition-colors disabled:opacity-50"
                        >
                          禁用
                        </button>
                      )}
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  )
}
