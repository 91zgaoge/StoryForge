# 网站侧会员系统设计文档（Admin 后台 + 邀请码发放 + JWT 吊销）

日期：2026-08-09
状态：已实现（W1–W9 全部落地，并入 v0.34.0；实现偏差见 ⑤ 邀请码格式）
范围：src-server Admin API + src-server-web 管理后台与用户 Dashboard；桌面端仅 expires_at 透传小改。付款仍不在本期。

## 背景

OAuth 订阅同步（v0.34.0 待发布）落地后，网站侧只有落地/登录/Dashboard 三个毛坯页面：
无管理员概念、无登录管理、邀请码只能 SQL 手插。本期补全会员系统。

## 关键决策（已与用户确认）

- 方案 A：扩展现有 src-server + src-server-web，零新基建
- 多管理员可管理（后台可提拔/降级）
- 邀请码可附带「注册即赠 Pro N 天」→ 顺带把 expires_at 生效逻辑做实（终审 backlog I2）
- 普通用户 Dashboard 显示订阅状态
- JWT 吊销本期一并修（backlog I3）：logout/禁用立即生效

## ① 数据模型与 Admin API（migration 004）

- `users` 加 `role TEXT NOT NULL DEFAULT 'user'`（user/admin）、`disabled_at TIMESTAMPTZ`
- `invite_codes` 加 `grant_pro_days INT`（NULL=不赠）、`created_by UUID`、`revoked_at TIMESTAMPTZ`（作废软删）
- 首个管理员：部署后手工 SQL `UPDATE users SET role='admin' WHERE email='...'`（文档给命令）
- Admin API（JWT + `require_admin` 提取器，非管理员 403）：
  - `GET /api/admin/users?q=` 用户列表（含 tier/role/状态/注册时间）
  - `POST /api/admin/users/{id}/role {role}` 提拔/降级（不能降级自己）
  - `POST /api/admin/users/{id}/disable` / `/enable`（禁用即删其 sessions = 立即踢下线）
  - `POST /api/admin/users/{id}/subscription {tier, days}` 手动赠/调订阅
  - `GET /api/admin/invite-codes`、`POST /api/admin/invite-codes {count, max_uses, grant_pro_days, note}`（批量生成，随机 8 位码）、`POST /api/admin/invite-codes/{code}/revoke`
- 所有 admin 写操作 `log::info!` 审计（操作者/动作/对象）

## ② JWT 吊销与禁用

- `AuthClaims::from_request` 验签后查 sessions 表（token 存在且未过期）+ `disabled_at IS NULL`，否则 401；主键索引查询不加缓存层
- logout 删 session 行（已有）→ 立即生效
- require_admin 每次查库校验 role（不信任 JWT claim，防旧 token 提权）

## ③ 邀请码赠 Pro + expires_at 生效

- 注册事务：码带 `grant_pro_days` → 同事务写 subscriptions（pro，expires_at=now+N 天）
- `GET /subscription/me`：`expires_at < now()` 的 pro 按 free 返回并懒更新库；`upsert_tier` 读回真实行（修 backlog #2）
- 桌面 `cache_remote_status` 透传 expires_at（修 backlog #12，本地不再恒写 30 天）

## ④ Web 前端（src-server-web）

- `/admin` 路由 + 守卫（非 admin 跳 Dashboard）；admin 的 Dashboard 顶部有「管理后台」入口
- 三页签：**邀请码**（生成表单：数量/次数/赠 Pro 天数/备注 + 列表：用量/状态/作废/复制）；**用户**（搜索 + 表格 + 行操作：赠 Pro 30/90/365 或改 free、禁用/启用）；**管理员**（列表 + 提拔/降级，不能操作自己）
- Dashboard 加「订阅状态」卡片（tier、到期时间；free 用户显示下载 app 升级引导）
- 技术沿用 axios + zustand + tailwind cinema 主题；新增 `src/api/admin.ts`、`src/pages/admin/`
- 用户角色获取：`GET /api/auth/me` 响应加 `role` 字段（前端据此显示/隐藏 admin 入口）

## ⑤ 安全 / 错误处理 / 测试

- 邀请码生成用密码学随机（自字母表 8 位，去易混字符）——**实现偏差**：实际为 `SM-` + 8 位大写 hex（取 UUID v4 前 8 位），撞 PK（code 唯一）兜底重试一次
- 非管理员 403；禁用/过期 401 前端清 token 跳登录；表单校验 400
- 测试：server sqlx::test——require_admin 拒普通用户、role 升降、禁用踢下线（sessions 删 + me 401）、邀请码生成/作废/赠 Pro 注册联动、`/me` 过期降级、upsert_tier 读回；web 前端补最小 vitest 基建 + 路由守卫与关键交互测试；桌面 cache_remote_status expires_at 透传单测
- 部署：migration 004 随 server 启动自动跑；SERVER_DEPLOYMENT.md 加「指定首个管理员」

## 同车清理的终审 backlog 项

- I2 expires_at 全链路生效（含 #2 upsert_tier 恒 active、#12 cache 丢 expires_at）
- I3 logout/禁用吊销 JWT
