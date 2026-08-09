import { Routes, Route } from 'react-router-dom'
import LandingPage from './pages/LandingPage'
import LoginPage from './pages/LoginPage'
import DashboardPage from './pages/DashboardPage'
import AdminLayout from './pages/admin/AdminLayout'
import InviteCodesPage from './pages/admin/InviteCodesPage'
import UsersPage from './pages/admin/UsersPage'
import AdminsPage from './pages/admin/AdminsPage'

function App() {
  return (
    <Routes>
      <Route path="/" element={<LandingPage />} />
      <Route path="/login" element={<LoginPage />} />
      <Route path="/dashboard" element={<DashboardPage />} />
      <Route path="/admin" element={<AdminLayout />}>
        <Route index element={<InviteCodesPage />} />
        <Route path="users" element={<UsersPage />} />
        <Route path="admins" element={<AdminsPage />} />
      </Route>
    </Routes>
  )
}

export default App
