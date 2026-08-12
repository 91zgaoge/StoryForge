# P0 幕后主题底座 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 幕后（backstage 窗口）从硬编码深色 cinema 色系改为 CSS 变量驱动 + 4 套深色调可选，与幕前色调主题（warm/cool/amber/indigo）同键双向同步，入口并入幕后设置页外观区。

**Architecture:** Tailwind 的 cinema 色已映射 `var(--cinema-*)`（tailwind.config.js:24-44），因此主题切换 = 运行时重写 documentElement 上的 `--cinema-*`/`--status-*`/`--border-subtle` 变量。新增 `backstageThemes.ts`（4 套深色调定义 + apply）与 `useBackstageTheme` hook（启动应用 + 监听 `color-theme-changed` Tauri 事件 / `storage` 事件），localStorage key 复用幕前 `storymoss-color-theme`，天然双向同步。

**Tech Stack:** React 18 + Tailwind v3.4（已配 `var()` 色）、zustand、vitest + Testing Library、Tauri event。

## Global Constraints

- 仓库 /Users/yuzaimu/projects/StoryForge；master 直接工作；中文 conventional commit；不 --no-verify；不推送、不打 tag。
- 不改 `tailwind.config.js`（cinema 已是 `var()` 引用）；不改幕前 `colorThemes.ts` 任何行为。
- 默认主题 warm 的色值必须与现状逐一相等（零视觉回归）。
- 不引入新依赖。
- 准入线：`cd src-frontend && npx tsc --noEmit && npx vitest run && npm run format:check` 全绿；vitest 基线 444 passed / 3 skipped，只允许增加。
- 设计文档：`docs/plans/2026-08-12-beautifului-ai-native-design.md`。

---

### Task 1: backstageThemes.ts — 4 套深色调 + apply

**Files:**
- Create: `src-frontend/src/styles/backstageThemes.ts`
- Test: `src-frontend/src/styles/__tests__/backstageThemes.test.ts`

**Interfaces:**
- Consumes: `ColorThemeId`（`@/frontstage/config/colorThemes`，'warm'|'cool'|'amber'|'indigo'）
- Produces:
  - `export const BACKSTAGE_THEME_VARS: readonly string[]` — 16 个变量名，供完整性测试与 apply 遍历
  - `export const backstageThemes: Record<ColorThemeId, BackstageTheme>`
  - `export function applyBackstageTheme(themeId: ColorThemeId): void` — 注入 documentElement
  - `export interface BackstageTheme { id: ColorThemeId; name: string; description: string; vars: Record<string,string> }`

- [ ] **Step 1: Write the failing test**

```typescript
// src-frontend/src/styles/__tests__/backstageThemes.test.ts
import { describe, it, expect } from 'vitest';
import {
  BACKSTAGE_THEME_VARS,
  backstageThemes,
  applyBackstageTheme,
} from '../backstageThemes';
import { colorThemeList } from '@/frontstage/config/colorThemes';

describe('backstageThemes', () => {
  it('每套主题覆盖全部 16 个必需变量，且选项与幕前色调同 id', () => {
    const ids = colorThemeList.map(t => t.id).sort();
    expect(Object.keys(backstageThemes).sort()).toEqual(ids);
    for (const theme of Object.values(backstageThemes)) {
      for (const key of BACKSTAGE_THEME_VARS) {
        expect(theme.vars[key], `${theme.id} 缺 ${key}`).toBeTruthy();
      }
    }
  });

  it('warm 主题与现状色值一致（零视觉回归）', () => {
    const warm = backstageThemes.warm.vars;
    expect(warm['--cinema-950']).toBe('#050508');
    expect(warm['--cinema-800']).toBe('#151520');
    expect(warm['--cinema-500']).toBe('#3a3a50');
    expect(warm['--cinema-gold']).toBe('#d4af37');
    expect(warm['--cinema-velvet']).toBe('#7c3aed');
    expect(warm['--status-success']).toBe('#22c55e');
  });

  it('applyBackstageTheme 注入全部变量到 documentElement', () => {
    applyBackstageTheme('cool');
    const style = document.documentElement.style;
    for (const key of BACKSTAGE_THEME_VARS) {
      expect(style.getPropertyValue(key)).toBe(backstageThemes.cool.vars[key]);
    }
    applyBackstageTheme('warm'); // 复位，避免污染其他测试
  });

  it('未知 id 回退 warm', () => {
    applyBackstageTheme('nope' as never);
    expect(document.documentElement.style.getPropertyValue('--cinema-gold')).toBe(
      backstageThemes.warm.vars['--cinema-gold']
    );
    applyBackstageTheme('warm');
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-frontend && npx vitest run src/styles/__tests__/backstageThemes.test.ts`
Expected: FAIL（模块不存在）

- [ ] **Step 3: Write implementation**

```typescript
// src-frontend/src/styles/backstageThemes.ts
/**
 * 幕后深色调主题系统（P0）
 *
 * Tailwind 的 cinema 色已映射 var(--cinema-*)（tailwind.config.js），
 * 主题切换 = 运行时重写 documentElement 上的同名变量。
 * 选项 id 与幕前色调主题（colorThemes.ts）一致：warm/cool/amber/indigo，
 * localStorage key 复用 storymoss-color-theme，天然双向同步。
 */
import type { ColorThemeId } from '@/frontstage/config/colorThemes';

export interface BackstageTheme {
  id: ColorThemeId;
  name: string;
  description: string;
  vars: Record<string, string>;
}

/** 每套主题必须给齐的变量（完整性测试遍历此表） */
export const BACKSTAGE_THEME_VARS = [
  '--cinema-950',
  '--cinema-900',
  '--cinema-850',
  '--cinema-800',
  '--cinema-700',
  '--cinema-600',
  '--cinema-500',
  '--cinema-gold',
  '--cinema-gold-light',
  '--cinema-gold-dark',
  '--cinema-velvet',
  '--status-success',
  '--status-success-dim',
  '--status-warning',
  '--status-danger',
  '--status-danger-dim',
] as const;

const STATUS = {
  '--status-success': '#22c55e',
  '--status-success-dim': 'rgba(34, 197, 94, 0.4)',
  '--status-warning': '#facc15',
  '--status-danger': '#ef4444',
  '--status-danger-dim': 'rgba(239, 68, 68, 0.4)',
};

export const backstageThemes: Record<ColorThemeId, BackstageTheme> = {
  warm: {
    id: 'warm',
    name: '暖金',
    description: '深色底 + 金色强调（默认，与现状一致）',
    vars: {
      '--cinema-950': '#050508',
      '--cinema-900': '#0a0a0f',
      '--cinema-850': '#0f0f16',
      '--cinema-800': '#151520',
      '--cinema-700': '#1e1e2e',
      '--cinema-600': '#2a2a3c',
      '--cinema-500': '#3a3a50',
      '--cinema-gold': '#d4af37',
      '--cinema-gold-light': '#e8c547',
      '--cinema-gold-dark': '#b8941f',
      '--cinema-velvet': '#7c3aed',
      ...STATUS,
    },
  },
  cool: {
    id: 'cool',
    name: '冷青',
    description: '深夜蓝底 + 青色强调，清新理性',
    vars: {
      '--cinema-950': '#04080c',
      '--cinema-900': '#081018',
      '--cinema-850': '#0b1620',
      '--cinema-800': '#101d29',
      '--cinema-700': '#162636',
      '--cinema-600': '#1f3347',
      '--cinema-500': '#2c455e',
      '--cinema-gold': '#22d3ee',
      '--cinema-gold-light': '#67e8f9',
      '--cinema-gold-dark': '#0891b2',
      '--cinema-velvet': '#38bdf8',
      ...STATUS,
    },
  },
  amber: {
    id: 'amber',
    name: '琥珀',
    description: '暖褐底 + 琥珀橙强调，温润古典',
    vars: {
      '--cinema-950': '#0a0705',
      '--cinema-900': '#120c07',
      '--cinema-850': '#181008',
      '--cinema-800': '#201609',
      '--cinema-700': '#2c1e0d',
      '--cinema-600': '#3d2a12',
      '--cinema-500': '#523a1b',
      '--cinema-gold': '#f59e0b',
      '--cinema-gold-light': '#fbbf24',
      '--cinema-gold-dark': '#d97706',
      '--cinema-velvet': '#fb923c',
      ...STATUS,
    },
  },
  indigo: {
    id: 'indigo',
    name: '靛紫',
    description: '紫夜底 + 靛蓝强调，沉静深邃',
    vars: {
      '--cinema-950': '#06060c',
      '--cinema-900': '#0b0b16',
      '--cinema-850': '#100f20',
      '--cinema-800': '#16152c',
      '--cinema-700': '#1d1b3a',
      '--cinema-600': '#282450',
      '--cinema-500': '#373266',
      '--cinema-gold': '#818cf8',
      '--cinema-gold-light': '#a5b4fc',
      '--cinema-gold-dark': '#6366f1',
      '--cinema-velvet': '#a78bfa',
      ...STATUS,
    },
  },
};

/** 将幕后主题应用到 documentElement；未知 id 回退 warm */
export function applyBackstageTheme(themeId: ColorThemeId) {
  const theme = backstageThemes[themeId] ?? backstageThemes.warm;
  const root = document.documentElement;
  for (const key of BACKSTAGE_THEME_VARS) {
    root.style.setProperty(key, theme.vars[key]);
  }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd src-frontend && npx vitest run src/styles/__tests__/backstageThemes.test.ts`
Expected: 4 passed

- [ ] **Step 5: Commit**

```bash
git add src-frontend/src/styles/backstageThemes.ts src-frontend/src/styles/__tests__/backstageThemes.test.ts
git commit -m "feat: 幕后深色调主题定义与 apply（P0 Task1）"
```

---

### Task 2: useBackstageTheme hook + App.tsx 全局接线

**Files:**
- Create: `src-frontend/src/hooks/useBackstageTheme.ts`
- Modify: `src-frontend/src/App.tsx`（在既有 agency 事件监听 useEffect 附近新增一个 effect；import 见下）
- Test: `src-frontend/src/hooks/__tests__/useBackstageTheme.test.ts`

**Interfaces:**
- Consumes: `applyBackstageTheme`（Task 1）；`loadColorTheme`/`COLOR_THEME_STORAGE_KEY`（`@/frontstage/config/colorThemes`）；Tauri `listen`（`@tauri-apps/api/event`）
- Produces: `export function useBackstageTheme(): void` — 挂载即应用当前主题，并持续监听两个同步通道

- [ ] **Step 1: Write the failing test**

```typescript
// src-frontend/src/hooks/__tests__/useBackstageTheme.test.ts
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { renderHook } from '@testing-library/react';
import { useBackstageTheme } from '../useBackstageTheme';
import { BACKSTAGE_THEME_VARS, backstageThemes } from '@/styles/backstageThemes';
import { COLOR_THEME_STORAGE_KEY } from '@/frontstage/config/colorThemes';

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}));

const readVars = () =>
  BACKSTAGE_THEME_VARS.map(k => document.documentElement.style.getPropertyValue(k));

describe('useBackstageTheme', () => {
  beforeEach(() => {
    localStorage.clear();
    for (const k of BACKSTAGE_THEME_VARS) document.documentElement.style.removeProperty(k);
  });

  it('挂载时按 localStorage 应用主题（无保存值 → warm）', () => {
    renderHook(() => useBackstageTheme());
    expect(readVars()).toEqual(BACKSTAGE_THEME_VARS.map(k => backstageThemes.warm.vars[k]));
  });

  it('localStorage 存了 cool → 挂载应用 cool', () => {
    localStorage.setItem(COLOR_THEME_STORAGE_KEY, 'cool');
    renderHook(() => useBackstageTheme());
    expect(readVars()).toEqual(BACKSTAGE_THEME_VARS.map(k => backstageThemes.cool.vars[k]));
  });

  it('storage 事件触发主题切换', () => {
    renderHook(() => useBackstageTheme());
    localStorage.setItem(COLOR_THEME_STORAGE_KEY, 'indigo');
    window.dispatchEvent(
      new StorageEvent('storage', { key: COLOR_THEME_STORAGE_KEY, newValue: 'indigo' })
    );
    expect(readVars()).toEqual(BACKSTAGE_THEME_VARS.map(k => backstageThemes.indigo.vars[k]));
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-frontend && npx vitest run src/hooks/__tests__/useBackstageTheme.test.ts`
Expected: FAIL（模块不存在）

- [ ] **Step 3: Write implementation**

```typescript
// src-frontend/src/hooks/useBackstageTheme.ts
/**
 * 幕后主题全局接线：挂载即应用当前色调对应的幕后深色调，
 * 并监听 storage / Tauri color-theme-changed 双通道实时切换。
 * 在幕后根组件（App.tsx）调用一次。
 */
import { useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import {
  COLOR_THEME_STORAGE_KEY,
  loadColorTheme,
  type ColorThemeId,
} from '@/frontstage/config/colorThemes';
import { applyBackstageTheme } from '@/styles/backstageThemes';

export function useBackstageTheme() {
  useEffect(() => {
    applyBackstageTheme(loadColorTheme());

    const handleStorage = (e: StorageEvent) => {
      if (e.key === COLOR_THEME_STORAGE_KEY || e.key === null) {
        applyBackstageTheme(loadColorTheme());
      }
    };
    window.addEventListener('storage', handleStorage);

    let unlisten: (() => void) | undefined;
    void listen<ColorThemeId>('color-theme-changed', event => {
      applyBackstageTheme(event.payload);
    })
      .then(fn => {
        unlisten = fn;
      })
      .catch(() => {
        /* non-Tauri / test env */
      });

    return () => {
      window.removeEventListener('storage', handleStorage);
      unlisten?.();
    };
  }, []);
}
```

在 `src-frontend/src/App.tsx`：
- 顶部 import 区加：`import { useBackstageTheme } from '@/hooks/useBackstageTheme';`
- `App` 组件内既有 hook 调用区（与其他 `useEffect` 同级、return JSX 之前）加一行：`useBackstageTheme();`

- [ ] **Step 4: Run tests**

Run: `cd src-frontend && npx vitest run src/hooks/__tests__/useBackstageTheme.test.ts && npx tsc --noEmit`
Expected: 3 passed；tsc 干净

- [ ] **Step 5: Commit**

```bash
git add src-frontend/src/hooks/useBackstageTheme.ts src-frontend/src/hooks/__tests__/useBackstageTheme.test.ts src-frontend/src/App.tsx
git commit -m "feat: 幕后主题全局接线——启动应用 + 双通道同步（P0 Task2）"
```

---

### Task 3: 设置页 ColorThemeSelector 双预览 + 应用幕后主题

**Files:**
- Modify: `src-frontend/src/pages/settings/GeneralSettings.tsx:42-110`（ColorThemeSelector）
- Test: 修改/新增 `src-frontend/src/pages/settings/__tests__/ColorThemeSelector.test.tsx`（若已有 GeneralSettings 测试则并入）

**Interfaces:**
- Consumes: `applyBackstageTheme`/`backstageThemes`（Task 1）；既有 `applyColorTheme`/`saveColorTheme`/`colorThemeList`
- Produces: 选择色调时同时应用幕前（applyColorTheme）与幕后（applyBackstageTheme）；每个选项渲染两个色点（幕前浅色调 + 幕后深色调）

- [ ] **Step 1: Write the failing test**

```typescript
// src-frontend/src/pages/settings/__tests__/ColorThemeSelector.test.tsx
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { ColorThemeSelector } from '../GeneralSettings';
import { BACKSTAGE_THEME_VARS, backstageThemes } from '@/styles/backstageThemes';
import { COLOR_THEME_STORAGE_KEY } from '@/frontstage/config/colorThemes';

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
  emit: vi.fn().mockResolvedValue(undefined),
}));

describe('ColorThemeSelector', () => {
  beforeEach(() => {
    localStorage.clear();
    for (const k of BACKSTAGE_THEME_VARS) document.documentElement.style.removeProperty(k);
  });

  it('选择 cool 后幕后 cinema 变量切换为 cool 深色调', () => {
    render(<ColorThemeSelector />);
    fireEvent.click(screen.getByText('冷青'));
    expect(document.documentElement.style.getPropertyValue('--cinema-gold')).toBe(
      backstageThemes.cool.vars['--cinema-gold']
    );
    expect(localStorage.getItem(COLOR_THEME_STORAGE_KEY)).toBe('cool');
  });

  it('每个选项渲染幕前/幕后双预览色点', () => {
    render(<ColorThemeSelector />);
    expect(screen.getAllByTestId(/theme-swatch-frontstage-/)).toHaveLength(4);
    expect(screen.getAllByTestId(/theme-swatch-backstage-/)).toHaveLength(4);
  });
});
```

注意：`ColorThemeSelector` 目前是 `GeneralSettings.tsx` 内的非导出函数——实现时给它加 `export`（单测需要，行为不变）。

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-frontend && npx vitest run src/pages/settings/__tests__/ColorThemeSelector.test.tsx`
Expected: FAIL（ColorThemeSelector 未导出 / data-testid 不存在）

- [ ] **Step 3: Write implementation**

`GeneralSettings.tsx` 改动：
1. import 区加：
```typescript
import { applyBackstageTheme, backstageThemes } from '@/styles/backstageThemes';
```
2. `function ColorThemeSelector()` 改为 `export function ColorThemeSelector()`。
3. `handleSelect` 与 `handleThemeChange` 里 `applyColorTheme(themeId)` 之后各加一行 `applyBackstageTheme(themeId);`。
4. JSX 中色点部分（原 L99-102 单个圆点）改为双预览：
```tsx
<div className="flex items-center gap-1">
  <div
    data-testid={`theme-swatch-frontstage-${theme.id}`}
    className="w-5 h-5 rounded-full border border-white/10"
    style={{ backgroundColor: theme.terracotta }}
    title="幕前色调"
  />
  <div
    data-testid={`theme-swatch-backstage-${theme.id}`}
    className="w-5 h-5 rounded-full border border-white/10"
    style={{ backgroundColor: backstageThemes[theme.id].vars['--cinema-gold'] }}
    title="幕后色调"
  />
</div>
```
5. 底部说明文案（原 L107）改为：`选择后即时生效，同步影响幕前写作界面与幕后工作台`。

- [ ] **Step 4: Run tests**

Run: `cd src-frontend && npx vitest run src/pages/settings && npx tsc --noEmit`
Expected: 全过（含既有 GeneralSettings 相关测试不回归）

- [ ] **Step 5: Commit**

```bash
git add src-frontend/src/pages/settings/GeneralSettings.tsx src-frontend/src/pages/settings/__tests__/ColorThemeSelector.test.tsx
git commit -m "feat: 设置页色调选择器双预览并同步应用幕后主题（P0 Task3）"
```

---

### Task 4: tokens.css 注释对齐 + 死代码清理

**Files:**
- Modify: `src-frontend/src/styles/tokens.css:15-26`（仅注释，不改值）
- Delete: `src-frontend/src/frontstage/hooks/useWritingStyle.ts`（死代码——全库无 import，真正生效路径是 `hooks/contracts/useEditorConfig.ts`）

- [ ] **Step 1: 确认 useWritingStyle 无引用**

Run: `cd src-frontend && grep -rn "useWritingStyle" src --include='*.ts*' | grep -v "frontstage/hooks/useWritingStyle.ts"`
Expected: 无输出（仅自身文件）。若有输出则停止本 Task，报告引用点。

- [ ] **Step 2: 修改 tokens.css 注释 + 删除死文件**

tokens.css 的 `/* 幕后「机械」 */` 注释行改为：
```css
  /* 幕后「机械」——值为 warm 主题默认值；运行时由 backstageThemes.ts 按色调重写 */
```
删除 `src-frontend/src/frontstage/hooks/useWritingStyle.ts`。

- [ ] **Step 3: Run full gate**

Run: `cd src-frontend && npx tsc --noEmit && npx vitest run 2>&1 | tail -3 && npm run format:check`
Expected: tsc 干净；vitest ≥ 451 passed / 3 skipped（基线 444 + Task1/2/3 新增）；format 通过

- [ ] **Step 4: Commit**

```bash
git add src-frontend/src/styles/tokens.css src-frontend/src/frontstage/hooks/useWritingStyle.ts
git commit -m "chore: tokens.css 主题注释对齐 + 删除 useWritingStyle 死代码（P0 Task4）"
```

---

## Self-Review 结论

- **Spec coverage**：设计文档 §4 幕后主题系统全部落地（变量化=本计划利用既有 var() 映射、4 套深色调=Task1、同步=Task2、入口=Task3、useWritingStyle 清理=Task4）。
- **Placeholder scan**：无 TBD/省略；每个代码步含完整代码。
- **Type consistency**：`applyBackstageTheme(themeId: ColorThemeId)`、`BACKSTAGE_THEME_VARS`、`backstageThemes` 在 Task1 定义，Task2/3 的 Consumes 一致引用；`ColorThemeSelector` 加 export 与测试 import 一致。
