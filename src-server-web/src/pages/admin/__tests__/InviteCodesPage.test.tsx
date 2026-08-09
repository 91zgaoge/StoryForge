import { render, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import InviteCodesPage from '../InviteCodesPage'
import { listInviteCodes, createInviteCodes, revokeInviteCode } from '../../../api/admin'

vi.mock('../../../api/admin', () => ({
  listInviteCodes: vi.fn(),
  createInviteCodes: vi.fn(),
  revokeInviteCode: vi.fn(),
}))

const mockedList = vi.mocked(listInviteCodes)
const mockedCreate = vi.mocked(createInviteCodes)
const mockedRevoke = vi.mocked(revokeInviteCode)

const activeCode = {
  code: 'ABC123',
  max_uses: 3,
  used_count: 1,
  grant_pro_days: 30,
  note: '活动发放',
  created_at: '2026-08-01T00:00:00Z',
  revoked_at: null,
}

const revokedCode = {
  code: 'XYZ789',
  max_uses: 1,
  used_count: 1,
  grant_pro_days: null,
  note: null,
  created_at: '2026-07-01T00:00:00Z',
  revoked_at: '2026-08-05T00:00:00Z',
}

describe('InviteCodesPage 邀请码管理', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('渲染邀请码列表（码/用量/赠 Pro/状态）', async () => {
    mockedList.mockResolvedValue([activeCode, revokedCode])
    render(<InviteCodesPage />)

    expect(await screen.findByText('ABC123')).toBeInTheDocument()
    expect(screen.getByText('XYZ789')).toBeInTheDocument()
    const row = screen.getByText('ABC123').closest('tr') as HTMLElement
    expect(within(row).getByText('1/3')).toBeInTheDocument()
    expect(within(row).getByText('30 天')).toBeInTheDocument()
    expect(within(row).getByText('活动发放')).toBeInTheDocument()
    expect(screen.getByText('生效中')).toBeInTheDocument()
    expect(screen.getByText('已作废')).toBeInTheDocument()
  })

  it('生成邀请码：提交参数并展示新码', async () => {
    mockedList.mockResolvedValue([])
    mockedCreate.mockResolvedValue({ codes: ['NEW-AAAA', 'NEW-BBBB'] })
    const user = userEvent.setup()
    render(<InviteCodesPage />)

    await user.clear(screen.getByLabelText('数量'))
    await user.type(screen.getByLabelText('数量'), '5')
    await user.selectOptions(screen.getByLabelText('赠 Pro 天数'), '30')
    await user.type(screen.getByLabelText('备注'), '内测批次')
    await user.click(screen.getByRole('button', { name: '生成邀请码' }))

    await waitFor(() =>
      expect(mockedCreate).toHaveBeenCalledWith({
        count: 5,
        max_uses: 1,
        grant_pro_days: 30,
        note: '内测批次',
      })
    )
    expect(await screen.findByText('NEW-AAAA')).toBeInTheDocument()
    expect(screen.getByText('NEW-BBBB')).toBeInTheDocument()
  })

  it('作废邀请码：确认后调用 revoke', async () => {
    mockedList.mockResolvedValue([activeCode])
    mockedRevoke.mockResolvedValue(undefined)
    vi.spyOn(window, 'confirm').mockReturnValue(true)
    const user = userEvent.setup()
    render(<InviteCodesPage />)

    await user.click(await screen.findByRole('button', { name: '作废' }))

    await waitFor(() => expect(mockedRevoke).toHaveBeenCalledWith('ABC123'))
  })
})
