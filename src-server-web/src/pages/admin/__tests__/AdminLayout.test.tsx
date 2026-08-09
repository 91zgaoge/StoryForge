import { render, screen } from '@testing-library/react'
import { MemoryRouter, Routes, Route } from 'react-router-dom'
import AdminLayout from '../AdminLayout'

describe('AdminLayout 守卫', () => {
  it('非 admin 跳回 /dashboard', () => {
    localStorage.setItem('sf_role', 'user')
    render(
      <MemoryRouter initialEntries={['/admin']}>
        <Routes>
          <Route path="/admin" element={<AdminLayout />}>
            <Route index element={<div>邀请码页</div>} />
          </Route>
          <Route path="/dashboard" element={<div>Dashboard 页</div>} />
        </Routes>
      </MemoryRouter>
    )
    expect(screen.getByText('Dashboard 页')).toBeInTheDocument()
    expect(screen.queryByText('邀请码页')).not.toBeInTheDocument()
  })

  it('admin 看到页签导航', () => {
    localStorage.setItem('sf_role', 'admin')
    render(
      <MemoryRouter initialEntries={['/admin']}>
        <Routes>
          <Route path="/admin" element={<AdminLayout />}>
            <Route index element={<div>邀请码页</div>} />
          </Route>
        </Routes>
      </MemoryRouter>
    )
    expect(screen.getByText('邀请码')).toBeInTheDocument()
    expect(screen.getByText('用户')).toBeInTheDocument()
    expect(screen.getByText('管理员')).toBeInTheDocument()
  })
})
