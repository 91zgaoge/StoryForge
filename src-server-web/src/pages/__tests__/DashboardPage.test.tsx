import { render, screen } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import axios from 'axios'
import DashboardPage from '../DashboardPage'
import { getMySubscription } from '../../api/subscription'

vi.mock('axios')
vi.mock('../../api/subscription', () => ({
  getMySubscription: vi.fn(),
}))

const mockedAxiosGet = vi.mocked(axios.get)
const mockedGetMySubscription = vi.mocked(getMySubscription)

function mockAuthMe(role: string) {
  mockedAxiosGet.mockResolvedValue({
    data: {
      id: 'u1',
      email: 'u@test.com',
      display_name: '测试用户',
      role,
    },
  })
}

describe('DashboardPage 订阅卡片', () => {
  beforeEach(() => {
    localStorage.clear()
    localStorage.setItem('sf_token', 'fake-token')
    vi.clearAllMocks()
  })

  it('pro 用户显示专业版与到期日期', async () => {
    mockAuthMe('user')
    mockedGetMySubscription.mockResolvedValue({
      tier: 'pro',
      status: 'active',
      expires_at: '2026-09-08T00:00:00Z',
    })
    render(
      <MemoryRouter initialEntries={['/dashboard']}>
        <DashboardPage />
      </MemoryRouter>
    )
    expect(await screen.findByText(/专业版/)).toBeInTheDocument()
    expect(screen.getByText(/2026-09-08/)).toBeInTheDocument()
  })

  it('free 用户显示升级引导文案', async () => {
    mockAuthMe('user')
    mockedGetMySubscription.mockResolvedValue({
      tier: 'free',
      status: 'active',
      expires_at: null,
    })
    render(
      <MemoryRouter initialEntries={['/dashboard']}>
        <DashboardPage />
      </MemoryRouter>
    )
    expect(await screen.findByText(/免费版/)).toBeInTheDocument()
    expect(screen.getByText(/升级/)).toBeInTheDocument()
  })
})
