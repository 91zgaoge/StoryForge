# OAuth 登录绑定订阅 + 跨设备同步 设计文档

日期：2026-08-09
状态：设计已获批准（头脑风暴流程），待制定实现计划
范围：真实注册/登录与订阅跨设备同步；**付款不在本期**

## 背景与目标

当前订阅（Pro tier）绑定设备 `.machine_id`，换设备即丢 Pro。src-tauri 已有完整但未打通的
OAuth 骨架（`auth/oauth.rs` PKCE 流程、V033 三张本地表、登录 UI），src-server 已有
Actix-web + Postgres 的 auth 服务（未部署、无订阅 API）。

目标：订阅绑定 OAuth 登录账号，登录后 Pro 跨设备同步；未登录沿用 machine_id 并提示
「登录以保留 Pro」。

## 关键决策（已与用户确认）

- 整体方案：**Server 为订阅权威源**（方案 A），桌面登录后订阅 user_id 切换为 server UUID
- 部署：storymoss.top 同机，Docker（docker-compose：Postgres + server）
- OAuth 提供商：Google + GitHub（骨架已完成；微信/QQ 预留不动）
- 桌面回调方式：**Server 中转 + 轮询**（不在桌面开端口、不用深链接）

## ① 架构与数据流

- 权威源：Postgres（src-server）存 `users` / `oauth_accounts` / `sessions` / 新增
  `subscriptions`；桌面本地 SQLite 只做缓存与离线降级
- 登录流程（server 中转）：
  1. 桌面点「Google 登录」→ `shell.open` 打开
     `https://storymoss.top/api/auth/google/start?client=desktop&state=<uuid>`
  2. server 完成 Google 回调、建用户与 session → 跳转「登录成功」页（含一次性兑换码）
  3. 桌面轮询 `GET /api/auth/desktop-poll?dstate=...` 拿 JWT → 存本地 sessions 表
- 订阅数据流：桌面所有订阅检查走 `SubscriptionBackend` 抽象——已登录 = RemoteBackend
  （打 server，结果写本地缓存）；未登录/离线 = LocalBackend（现状 machine_id 逻辑）

## ② Server 端（src-server）

- 新迁移 `002_subscriptions.sql`：
  `subscriptions(id, user_id UNIQUE FK, tier, status, started_at, expires_at, source,
  created_at, updated_at)`（一对一）
- 新 API：
  - `GET /api/subscription/me`（JWT）→ `{ tier, status, expires_at }`，无记录自动建 free
  - `POST /api/subscription/dev-upgrade { tier }`（JWT；接付款后改为支付回调写库；
    生产可用 env 开关关闭）
  - `GET /api/auth/desktop-poll?dstate=...`：桌面轮询登录结果——dstate 一次性
    （取到即删）、10 分钟 TTL；202 pending / 200 返回 JWT+user / 403 返回失败码
- OAuth start 支持 `client=desktop`，callback 按 client 决定跳 Web dashboard 或展示
  兑换码页
- 复用现有 Google/GitHub userinfo 骨架

### ②B 邀请码注册门控（2026-08-09 追加需求）

- 内测期门控：OAuth 登录时**新用户注册**必须持有效邀请码；老用户（OAuth 账号或
  email 已存在）免码直接登录
- `invite_codes(code PK, max_uses DEFAULT 1, used_count, note)`；原子占码
  （`UPDATE ... WHERE used_count < max_uses` 受影响行数判断），校验+建用户+计数同事务
- start 增加可选 `invite` 参数随 state 透传；无码/错码/用满 → desktop 流程展示错误
  HTML 页，其余 403 `invalid_or_used_invite`
- 桌面登录窗加邀请码输入框随 `oauth_start` 传递
- 发码：管理员 SQL 手工插入（`max_uses` 可配多次），暂不做管理界面

## ③ 桌面端（src-tauri）

- 打通 auth 骨架：`oauth_start` 改为「打开 server 授权页 + 后台轮询 exchange」；
  `oauth_callback` 废弃；`get_current_user` 改为读本地 session 返回真实用户；
  `logout` 删本地 session 并通知 server
- JWT 存本地 `sessions` 表；reqwest 带 `Authorization: Bearer`
- 订阅 user_id 收口：新增 `SubscriptionBackend` trait + Remote/Local 实现；
  6 处散落的 `get_user_id()`（subscription/agents/pipeline/book_deconstruction/
  guidebook_distillation/llm）统一为 `resolve_subscription_identity()`——已登录返回
  server UUID + RemoteBackend，否则 machine_id + LocalBackend
- 本地缓存：Remote 结果按 tier-diff 落本地 `subscriptions` 缓存（tier 变化才插新行，
  无新列/新迁移）；离线时 `has_feature_access` 用缓存 tier；已登录 dev 升级写 server
  成功后更新本地

## ④ 前端 UI（src-frontend）

- 接通现有 `LoginModal` / `UserMenu` / `AccountSettings`；登录中显示「等待浏览器授权…」
  轮询态，2 分钟超时可取消
- `UpgradeModal`：未登录 → 引导登录（「登录后升级，Pro 跟随账号」）+「暂不登录，仅本设备
  升级」；已登录 → RemoteBackend 升级
- 账户页显示订阅来源（账号同步 / 仅本设备）

## ⑤ 部署与 CI

- docker-compose：postgres + server；server 加 `/healthz`；storymoss.top 主机 Nginx
  反代 `/api/` → server:8080，静态站与 latest.json 路径不变
- 新工作流 `.github/workflows/deploy-server.yml`：SSH 到主机 `docker compose up -d`
  （secrets 存 GitHub）
- Google/GitHub 各注册一个 OAuth App，callback 指向
  `https://storymoss.top/api/auth/{provider}/callback`，client_id/secret 进主机 env

## ⑥ 错误处理与测试

- 网络失败 → 静默降级本地缓存（不打断写作）；JWT 过期 → 提示重新登录；server 5xx →
  本地缓存 + 日志；兑换码过期 → 重新发起登录
- 安全：兑换码一次性 60s；JWT 7 天（沿用）
- 测试：server 集成测试（subscription API + exchange 一次性）；桌面 Local/RemoteBackend
  单测 + 身份切换测试；前端升级链路扩到未登录/已登录两路径；手动验证双设备同账号
  Pro 同步

## 现状备忘（实施时参考）

- 三套用户体系：本地 users（V033）、machine_id（订阅实际使用）、server users——核心工作
  是把订阅 user_id 从 machine_id 切到 server UUID，未登录降级 machine_id
- `auth/oauth.rs` 只探测端口未起监听——本期用 server 中转，该本地监听不实现
- `get_current_user` 目前是占位实现恒返回 None
- tauri capabilities：`shell:allow-open` 已有；`http:default` 仅 localhost，Rust reqwest
  调远端不受限；CSP `connect-src *` 无需改
- CI 无 server 部署工作流；server 无订阅 API；landing 纯静态 FTP 部署不受影响
