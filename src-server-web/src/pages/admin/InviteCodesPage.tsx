import { useEffect, useState } from 'react'
import { Ticket, Copy, Loader2 } from 'lucide-react'
import {
  listInviteCodes,
  createInviteCodes,
  revokeInviteCode,
  type InviteCode,
} from '../../api/admin'

const GRANT_PRO_OPTIONS = [
  { value: 0, label: '不赠' },
  { value: 7, label: '7 天' },
  { value: 30, label: '30 天' },
  { value: 90, label: '90 天' },
  { value: 365, label: '365 天' },
]

function formatDate(iso: string): string {
  return new Date(iso).toISOString().slice(0, 10)
}

export default function InviteCodesPage() {
  const [codes, setCodes] = useState<InviteCode[]>([])
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState('')

  const [count, setCount] = useState(10)
  const [maxUses, setMaxUses] = useState(1)
  const [grantProDays, setGrantProDays] = useState(0)
  const [note, setNote] = useState('')
  const [isCreating, setIsCreating] = useState(false)
  const [newCodes, setNewCodes] = useState<string[]>([])
  const [toast, setToast] = useState('')

  const refresh = () => {
    listInviteCodes()
      .then(setCodes)
      .catch(() => setError('加载邀请码失败'))
      .finally(() => setIsLoading(false))
  }

  useEffect(refresh, [])

  const handleCreate = async () => {
    setIsCreating(true)
    setToast('')
    try {
      const res = await createInviteCodes({
        count,
        max_uses: maxUses,
        ...(grantProDays > 0 ? { grant_pro_days: grantProDays } : {}),
        ...(note.trim() ? { note: note.trim() } : {}),
      })
      setNewCodes(res.codes)
      setToast(`已生成 ${res.codes.length} 个邀请码`)
      refresh()
    } catch {
      setToast('生成失败，请重试')
    } finally {
      setIsCreating(false)
    }
  }

  const handleRevoke = async (code: string) => {
    if (!window.confirm(`确认作废邀请码 ${code}？`)) return
    try {
      await revokeInviteCode(code)
      refresh()
    } catch {
      setToast('作废失败，请重试')
    }
  }

  const handleCopy = (code: string) => {
    navigator.clipboard.writeText(code).catch(() => {})
  }

  return (
    <div className="space-y-6">
      {/* 生成表单 */}
      <div className="bg-cinema-900/50 border border-cinema-800/50 rounded-2xl p-6">
        <h2 className="text-lg font-semibold text-white flex items-center gap-2">
          <Ticket className="w-5 h-5 text-cinema-gold" />
          生成邀请码
        </h2>
        <div className="mt-4 grid grid-cols-2 md:grid-cols-4 gap-4">
          <div>
            <label htmlFor="invite-count" className="block text-xs text-gray-400 mb-1">
              数量
            </label>
            <input
              id="invite-count"
              type="number"
              min={1}
              max={100}
              value={count}
              onChange={e => setCount(Number(e.target.value))}
              className="w-full px-3 py-2 bg-cinema-800/50 border border-cinema-700/50 rounded-lg text-sm text-white focus:outline-none focus:border-cinema-gold/50"
            />
          </div>
          <div>
            <label htmlFor="invite-max-uses" className="block text-xs text-gray-400 mb-1">
              每码可用次数
            </label>
            <input
              id="invite-max-uses"
              type="number"
              min={1}
              value={maxUses}
              onChange={e => setMaxUses(Number(e.target.value))}
              className="w-full px-3 py-2 bg-cinema-800/50 border border-cinema-700/50 rounded-lg text-sm text-white focus:outline-none focus:border-cinema-gold/50"
            />
          </div>
          <div>
            <label htmlFor="invite-grant-pro" className="block text-xs text-gray-400 mb-1">
              赠 Pro 天数
            </label>
            <select
              id="invite-grant-pro"
              value={grantProDays}
              onChange={e => setGrantProDays(Number(e.target.value))}
              className="w-full px-3 py-2 bg-cinema-800/50 border border-cinema-700/50 rounded-lg text-sm text-white focus:outline-none focus:border-cinema-gold/50"
            >
              {GRANT_PRO_OPTIONS.map(opt => (
                <option key={opt.value} value={opt.value}>
                  {opt.label}
                </option>
              ))}
            </select>
          </div>
          <div>
            <label htmlFor="invite-note" className="block text-xs text-gray-400 mb-1">
              备注
            </label>
            <input
              id="invite-note"
              type="text"
              value={note}
              onChange={e => setNote(e.target.value)}
              className="w-full px-3 py-2 bg-cinema-800/50 border border-cinema-700/50 rounded-lg text-sm text-white focus:outline-none focus:border-cinema-gold/50"
            />
          </div>
        </div>
        <div className="mt-4 flex items-center gap-4">
          <button
            onClick={handleCreate}
            disabled={isCreating}
            className="flex items-center gap-2 px-4 py-2 bg-cinema-gold text-cinema-900 rounded-lg text-sm font-medium hover:bg-cinema-gold-light transition-colors disabled:opacity-50"
          >
            {isCreating && <Loader2 className="w-4 h-4 animate-spin" />}
            生成邀请码
          </button>
          {toast && <span className="text-sm text-gray-400">{toast}</span>}
        </div>

        {/* 新生成的码 */}
        {newCodes.length > 0 && (
          <div className="mt-4 p-4 rounded-xl bg-cinema-800/30 border border-cinema-700/30">
            <p className="text-xs text-gray-400 mb-2">新生成的邀请码</p>
            <div className="flex flex-wrap gap-2">
              {newCodes.map(code => (
                <span
                  key={code}
                  className="flex items-center gap-2 px-3 py-1.5 bg-cinema-900/60 border border-cinema-700/40 rounded-lg"
                >
                  <span className="font-mono text-sm text-white">{code}</span>
                  <button
                    onClick={() => handleCopy(code)}
                    title="复制"
                    className="text-gray-400 hover:text-white transition-colors"
                  >
                    <Copy className="w-3.5 h-3.5" />
                  </button>
                </span>
              ))}
            </div>
          </div>
        )}
      </div>

      {/* 列表 */}
      <div className="bg-cinema-900/50 border border-cinema-800/50 rounded-2xl p-6">
        <h2 className="text-lg font-semibold text-white">邀请码列表</h2>
        {isLoading ? (
          <p className="text-sm text-gray-400 mt-4">加载中...</p>
        ) : error ? (
          <p className="text-sm text-red-400 mt-4">{error}</p>
        ) : codes.length === 0 ? (
          <p className="text-sm text-gray-400 mt-4">暂无邀请码</p>
        ) : (
          <div className="mt-4 overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="text-left text-xs text-gray-500 border-b border-cinema-800/50">
                  <th className="pb-2 pr-4">码</th>
                  <th className="pb-2 pr-4">用量</th>
                  <th className="pb-2 pr-4">赠 Pro</th>
                  <th className="pb-2 pr-4">备注</th>
                  <th className="pb-2 pr-4">状态</th>
                  <th className="pb-2 pr-4">创建时间</th>
                  <th className="pb-2">操作</th>
                </tr>
              </thead>
              <tbody>
                {codes.map(c => (
                  <tr key={c.code} className="border-b border-cinema-800/30">
                    <td className="py-3 pr-4 font-mono text-white">{c.code}</td>
                    <td className="py-3 pr-4 text-gray-300">
                      {c.used_count}/{c.max_uses}
                    </td>
                    <td className="py-3 pr-4 text-gray-300">
                      {c.grant_pro_days ? `${c.grant_pro_days} 天` : '—'}
                    </td>
                    <td className="py-3 pr-4 text-gray-300">{c.note || '—'}</td>
                    <td className="py-3 pr-4">
                      {c.revoked_at ? (
                        <span className="text-red-400">已作废</span>
                      ) : (
                        <span className="text-green-400">生效中</span>
                      )}
                    </td>
                    <td className="py-3 pr-4 text-gray-400">{formatDate(c.created_at)}</td>
                    <td className="py-3">
                      {!c.revoked_at && (
                        <button
                          onClick={() => handleRevoke(c.code)}
                          className="text-red-400 hover:text-red-300 text-sm transition-colors"
                        >
                          作废
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
