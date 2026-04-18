<!-- SPECKIT START -->
For additional context about technologies to be used, project structure,
shell commands, and other important information, read the current plan.

## Project Context

**StoryForge (草苔)** — AI-assisted novel writing desktop application
- GitHub: https://github.com/91zgaoge/StoryForge
- Version: v3.3.0
- Tech Stack: Tauri 2.4 + Rust 1.94 + React 18 + TypeScript 5.8 + SQLite

### Architecture
- **Frontstage (幕前)**: Immersive writing interface (`/frontstage.html`) — warm paper tone, distraction-free
- **Backstage (幕后)**: Studio management (`/index.html`) — dark Cinema theme
- **Backend**: Rust with Tauri IPC, SQLite (rusqlite + r2d2), LanceDB vectors
- **Data Layer**: All user data stored locally; supports local LLM (Ollama) + cloud providers

### Key Development Commands
```powershell
# Frontend dev server
cd src-frontend && npm run dev

# Tauri dev
cd src-tauri && cargo tauri dev

# Release build (MUST pass before push)
cd src-tauri && cargo tauri build

# Tests
cargo test
npm test
```

### Critical Rules
1. **Build before push**: `cargo check` (0 warnings) → `cargo test` → `npm run build` → `cargo tauri build`
2. **Version sync**: Cargo.toml, tauri.conf.json, package.json, package-lock.json must match
3. **Minimal changes**: Extend existing patterns; snake_case (Rust), camelCase (TS)
4. **Zero warnings policy**: `cargo check` must produce zero warnings

### Important Documents
- `.specify/memory/constitution.md` — Project constitution (Spec-Kit)
- `AGENTS.md` — Agent development guide
- `ARCHITECTURE.md` — System architecture
- `ROADMAP.md` — Development roadmap
- `CHANGELOG.md` — Release changelog
<!-- SPECKIT END -->
