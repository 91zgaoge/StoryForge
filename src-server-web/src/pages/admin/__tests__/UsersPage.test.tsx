import { render, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import UsersPage from '../UsersPage'
import {
  listUsers,
  disableUser,
  enableUser,
  setUserSubscription,
} from '../../../api/admin'

vi.mock('../../../api/admin', () => ({
  listUsers: vi.fn(),
  setUserRole: vi.fn(),
  disableUser: vi.fn(),
  enableUser: vi.fn(),
  setUserSubscription: vi.fn(),
}))

const mockedList = vi.mocked(listUsers)
const mockedDisable = vi.mocked(disableUser)
const mockedEnable = vi.mocked(enableUser)
const mockedSetSubscription = vi.mocked(setUserSubscription)

const adminUser = {
  id: 'u-admin',
  email: 'admin@test.com',
  display_name: '管理员甲',
  role: 'admin',
  tier: 'pro',
  expires_at: '2026-12-31T00:00:00Z',
  disabled_at: null,
  created_at: '2026-01-01T00:00:00Z',
}

const normalUser = {
  id: 'u-user',
  email: 'user@test.com',
  display_name: '用户乙',
  role: 'user',
  tier: null,
  expires_at: null,
  disabled_at: null,
  created_at: '2026-07-01T00:00:00Z',
}

function rowOf(text: string): HTMLElement {
  return screen.getByText(text).closest('tr') as HTMLElement
}

describe('UsersPage 用户管理', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('渲染用户列表（邮箱/昵称/tier/角色/状态）', async () => {
    mockedList.mockResolvedValue([adminUser, normalUser])
    render(<UsersPage />)

    expect(await screen.findByText('admin@test.com')).toBeInTheDocument()
    expect(screen.getByText('user@test.com')).toBeInTheDocument()
    const adminRow = rowOf('admin@test.com')
    expect(within(adminRow).getByText('管理员甲')).toBeInTheDocument()
    expect(within(adminRow).getByText('Pro')).toBeInTheDocument()
    expect(within(adminRow).getByText('管理员')).toBeInTheDocument()
    expect(within(adminRow).getByText('正常')).toBeInTheDocument()
    const userRow = rowOf('user@test.com')
    expect(within(userRow).getByText('用户乙')).toBeInTheDocument()
    expect(within(userRow).getByText('免费')).toBeInTheDocument()
    expect(within(userRow).getByText('用户')).toBeInTheDocument()
  })

  it('禁用用户显示「禁用」状态与「启用」操作', async () => {
    mockedList.mockResolvedValue([{ ...normalUser, disabled_at: '2026-08-01T00:00:00Z' }])
    render(<UsersPage />)

    await screen.findByText('user@test.com')
    const row = rowOf('user@test.com')
    expect(within(row).getByText('禁用')).toBeInTheDocument()
    expect(within(row).getByRole('button', { name: '启用' })).toBeInTheDocument()
  })

  it('赠 Pro 30 天：调用 setUserSubscription(id, pro, 30)', async () => {
    mockedList.mockResolvedValue([normalUser])
    mockedSetSubscription.mockResolvedValue(undefined)
    const user = userEvent.setup()
    render(<UsersPage />)

    await screen.findByText('user@test.com')
    const row = rowOf('user@test.com')
    await user.click(within(row).getByRole('button', { name: '赠 Pro 30 天' }))

    await waitFor(() =>
      expect(mockedSetSubscription).toHaveBeenCalledWith('u-user', 'pro', 30)
    )
  })

  it('禁用：二次确认后调用 disableUser 并刷新', async () => {
    mockedList.mockResolvedValue([normalUser])
    mockedDisable.mockResolvedValue(undefined)
    vi.spyOn(window, 'confirm').mockReturnValue(true)
    const user = userEvent.setup()
    render(<UsersPage />)

    await screen.findByText('user@test.com')
    const row = rowOf('user@test.com')
    await user.click(within(row).getByRole('button', { name: '禁用' }))

    await waitFor(() => expect(mockedDisable).toHaveBeenCalledWith('u-user'))
    expect(mockedEnable).not.toHaveBeenCalled()
  })

  it('禁用：取消确认则不调用 disableUser', async () => {
    mockedList.mockResolvedValue([normalUser])
    vi.spyOn(window, 'confirm').mockReturnValue(false)
    const user = userEvent.setup()
    render(<UsersPage />)

    await screen.findByText('user@test.com')
    const row = rowOf('user@test.com')
    await user.click(within(row).getByRole('button', { name: '禁用' }))

    expect(mockedDisable).not.toHaveBeenCalled()
  })

  it('搜索框输入触发 listUsers(q)', async () => {
    mockedList.mockResolvedValue([normalUser])
    const user = userEvent.setup()
    render(<UsersPage />)

    await screen.findByText('user@test.com')
    await user.type(screen.getByLabelText('搜索用户'), 'user@test')

    await waitFor(() =>
      expect(mockedList).toHaveBeenLastCalledWith('user@test')
    )
  })
})
