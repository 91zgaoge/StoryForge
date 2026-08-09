import { Navigate, NavLink, Outlet, Link } from 'react-router-dom'
import { Shield, ArrowLeft } from 'lucide-react'

export function getRole(): string | null {
  return localStorage.getItem('sf_role')
}

const tabs = [
  { to: '/admin', label: '邀请码', end: true },
  { to: '/admin/users', label: '用户', end: false },
  { to: '/admin/admins', label: '管理员', end: false },
]

export default function AdminLayout() {
  if (getRole() !== 'admin') {
    return <Navigate to="/dashboard" replace />
  }

  return (
    <div className="min-h-screen bg-cinema-950">
      {/* Header */}
      <header className="border-b border-cinema-800/50 bg-cinema-900/50">
        <div className="max-w-6xl mx-auto px-6 h-16 flex items-center justify-between">
          <div className="flex items-center gap-2 text-white">
            <Shield className="w-5 h-5 text-cinema-gold" />
            <span className="font-display font-bold">管理后台</span>
          </div>
          <Link
            to="/dashboard"
            className="flex items-center gap-2 text-sm text-gray-400 hover:text-white transition-colors"
          >
            <ArrowLeft className="w-4 h-4" />
            返回 Dashboard
          </Link>
        </div>
      </header>

      {/* Tabs */}
      <nav className="border-b border-cinema-800/50">
        <div className="max-w-6xl mx-auto px-6 flex gap-2">
          {tabs.map(tab => (
            <NavLink
              key={tab.to}
              to={tab.to}
              end={tab.end}
              className={({ isActive }) =>
                `px-4 py-3 text-sm font-medium border-b-2 transition-colors ${
                  isActive
                    ? 'border-cinema-gold text-white'
                    : 'border-transparent text-gray-400 hover:text-white'
                }`
              }
            >
              {tab.label}
            </NavLink>
          ))}
        </div>
      </nav>

      {/* Content */}
      <main className="max-w-6xl mx-auto px-6 py-8">
        <Outlet />
      </main>
    </div>
  )
}
