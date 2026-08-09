# StoryMoss Server 部署指南

## 概述

StoryMoss Server v4.5.0 包含三个组件：

| 组件             | 技术             | 端口 | 说明                         |
| ---------------- | ---------------- | ---- | ---------------------------- |
| PostgreSQL       | 数据库           | 5432 | 用户/会话数据持久化          |
| StoryMoss Server | Actix-web (Rust) | 8080 | REST API + OAuth             |
| StoryMoss Web    | React + Nginx    | 80   | 落地页 + Web登录 + Dashboard |

## 快速开始（Docker Compose）

### 1. 准备环境

```bash
# 安装 Docker + Docker Compose
# https://docs.docker.com/get-docker/

# 克隆项目
git clone https://github.com/91zgaoge/StoryMoss.git
cd StoryMoss
```

### 2. 配置环境变量

```bash
cp .env.example .env
nano .env  # 编辑配置
```

必须配置的项：

- `POSTGRES_PASSWORD` — 数据库密码
- `JWT_SECRET` — JWT签名密钥（至少32字符）
- `GOOGLE_CLIENT_ID` / `GOOGLE_CLIENT_SECRET` — 或 GitHub OAuth

### 3. 启动服务

```bash
docker-compose up -d
```

### 4. 验证

```bash
# 检查健康状态
curl http://localhost:8080/api/health

# 访问落地页
open http://localhost
```

## OAuth 应用注册

### Google

1. 访问 https://console.cloud.google.com/apis/credentials
2. 创建 OAuth 2.0 客户端 ID
3. 应用类型选择 "Web 应用"
4. 授权重定向 URI: `http://your-domain/api/auth/google/callback`
5. 复制客户端 ID 和密钥到 `.env`

### GitHub

1. 访问 https://github.com/settings/developers
2. 新建 OAuth App
3. Authorization callback URL: `http://your-domain/api/auth/github/callback`
4. 复制 Client ID 和 Client Secret 到 `.env`

### 微信/QQ（预留）

需要在对应的开放平台注册应用，配置方式类似。

## 目录结构

```
StoryMoss/
├── src-tauri/           # 桌面端（Tauri + Rust）
│   └── src/auth/        # 桌面端认证模块
├── src-frontend/        # 桌面端前端（React）
├── src-server/          # 【服务端后端】
│   ├── src/
│   │   ├── main.rs      # Actix-web 入口
│   │   ├── config.rs    # 环境配置
│   │   ├── auth/        # OAuth + JWT
│   │   └── api/         # REST API
│   ├── migrations/      # PostgreSQL 迁移
│   └── Dockerfile
├── src-server-web/      # 【服务端前端】
│   ├── src/pages/
│   │   ├── LandingPage.tsx
│   │   ├── LoginPage.tsx
│   │   └── DashboardPage.tsx
│   └── Dockerfile
├── docker-compose.yml   # 一键部署
└── .env.example         # 配置模板
```

## API 端点

| 方法 | 路径                          | 说明                  |
| ---- | ----------------------------- | --------------------- |
| GET  | /api/health                   | 健康检查              |
| GET  | /api/auth/config              | 获取已启用的OAuth配置 |
| GET  | /api/auth/{provider}/start    | 开始OAuth登录         |
| GET  | /api/auth/{provider}/callback | OAuth回调             |
| POST | /api/auth/logout              | 注销                  |
| GET  | /api/auth/me                  | 获取当前用户          |
| GET  | /api/users/me                 | 获取当前用户详情      |

## 更新

```bash
# 拉取最新代码
git pull origin main

# 重建并重启
docker-compose down
docker-compose up -d --build
```

## 生产部署（storymoss.top）

### Nginx 反向代理

server 容器仅绑定 `127.0.0.1:8080`（见 `docker-compose.yml`），由主机 nginx 反代 `/api/`：

```nginx
location /api/ {
    proxy_pass http://127.0.0.1:8080;
    proxy_set_header Host $host;
}
```

web 服务与 storymoss.top 静态站的共存方式以主机实际 Web 服务为准。

### 上线手工清单

1. **Google Cloud Console**（https://console.cloud.google.com/apis/credentials）创建 OAuth client，callback 填 `https://storymoss.top/api/auth/google/callback`。
2. **GitHub** Settings → Developer settings → OAuth Apps 新建应用，callback 填 `https://storymoss.top/api/auth/github/callback`。
3. 主机 `/opt/storymoss/.env` 填写 `GOOGLE_CLIENT_ID` / `GOOGLE_CLIENT_SECRET`、`GITHUB_CLIENT_ID` / `GITHUB_CLIENT_SECRET`、`JWT_SECRET`（与桌面端无关，server 自签自验）、`POSTGRES_PASSWORD`、`FRONTEND_URL=https://storymoss.top`、`SERVER_BASE_URL=https://storymoss.top`。注意 `SERVER_BASE_URL` 必须与第 1、2 步在 OAuth App 后台注册的 callback 前缀一致（即 nginx 反代对外域名），否则登录会因 redirect_uri 不匹配而失败。
4. GitHub 仓库 Settings → Secrets 配置 `SERVER_SSH_HOST` / `SERVER_SSH_USER` / `SERVER_SSH_KEY`（供 `.github/workflows/deploy-server.yml` 使用）。
5. 首次部署：主机上 `cd /opt/storymoss && docker compose up -d`；之后可通过 Actions 页面手动触发 **Deploy Server** 工作流。

### 邀请码发放

新用户注册需有效邀请码（老用户免码）。在主机上发放：

```bash
docker compose exec postgres psql -U storymoss -c "INSERT INTO invite_codes (code, max_uses, note) VALUES ('BETA-XXXX', 1, '发给某某');"
```

查询余量：

```bash
docker compose exec postgres psql -U storymoss -c "SELECT code, used_count, max_uses FROM invite_codes;"
```
