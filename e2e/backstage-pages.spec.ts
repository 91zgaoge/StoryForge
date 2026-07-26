import { test, expect } from '@playwright/test';
import { getMockTauriInitScript } from './mock-tauri';

/**
 * Backstage 各页面加载测试
 * 验证每个页面能正确渲染且无控制台报错
 */
test.describe('Backstage 页面加载测试', () => {
  test.beforeEach(async ({ page }) => {
    await page.setViewportSize({ width: 1920, height: 1080 });
  });

  test('仪表盘页面加载无报错', async ({ page }) => {
    const consoleErrors: string[] = [];
    page.on('console', msg => {
      if (msg.type() === 'error') consoleErrors.push(msg.text());
    });
    page.on('pageerror', err => consoleErrors.push(err.message));

    await page.addInitScript(getMockTauriInitScript());
    await page.goto('/index.html');

    await expect(page.locator('aside')).toBeVisible({ timeout: 10000 });
    await expect(page.locator('main')).toBeVisible();

    // 仪表盘应正常渲染
    expect(consoleErrors.filter(e => !e.includes('enablePersistence'))).toHaveLength(0);
  });

  test('故事页面加载并显示故事列表', async ({ page }) => {
    const consoleErrors: string[] = [];
    page.on('console', msg => {
      if (msg.type() === 'error') consoleErrors.push(msg.text());
    });
    page.on('pageerror', err => consoleErrors.push(err.message));

    await page.addInitScript(getMockTauriInitScript());
    await page.goto('/index.html');

    // 点击故事导航
    await page.locator('nav').locator('text=故事').first().click();
    await page.waitForTimeout(1000);

    await expect(page.locator('h1')).toContainText('故事库');

    // 断言至少有一个故事卡片或空状态提示
    const storyCards = page.locator('h3');
    await expect(storyCards.first()).toBeVisible();

    expect(consoleErrors.filter(e => !e.includes('enablePersistence'))).toHaveLength(0);
  });

  test('角色页面加载并显示列表', async ({ page }) => {
    const consoleErrors: string[] = [];
    page.on('console', msg => {
      if (msg.type() === 'error') consoleErrors.push(msg.text());
    });
    page.on('pageerror', err => consoleErrors.push(err.message));

    await page.addInitScript(getMockTauriInitScript());
    await page.goto('/index.html');

    // 点击角色导航
    await page.locator('nav').locator('text=角色').first().click();
    await page.waitForTimeout(1000);

    // 角色页面在故事已选择时显示角色管理
    await expect(page.locator('main')).toContainText('角色管理');

    expect(consoleErrors.filter(e => !e.includes('enablePersistence'))).toHaveLength(0);
  });

  test('场景页面加载并显示场景管理', async ({ page }) => {
    const consoleErrors: string[] = [];
    page.on('console', msg => {
      if (msg.type() === 'error') consoleErrors.push(msg.text());
    });
    page.on('pageerror', err => consoleErrors.push(err.message));

    await page.addInitScript(getMockTauriInitScript());
    await page.goto('/index.html');

    // 点击场景导航
    await page.locator('nav').locator('text=场景').first().click();
    await page.waitForTimeout(1200);

    // 场景页面在故事已选择时显示场景管理界面
    await expect(page.locator('body')).toContainText('选择一个场景');

    expect(consoleErrors.filter(e => !e.includes('enablePersistence'))).toHaveLength(0);
  });

  test('设置页面加载并显示所有标签页', async ({ page }) => {
    const consoleErrors: string[] = [];
    page.on('console', msg => {
      if (msg.type() === 'error') consoleErrors.push(msg.text());
    });
    page.on('pageerror', err => consoleErrors.push(err.message));

    await page.addInitScript(getMockTauriInitScript());
    await page.goto('/index.html');

    // 点击设置导航
    await page.locator('nav').locator('text=设置').first().click();
    await page.waitForTimeout(1000);

    await expect(page.locator('h1')).toContainText('工作室配置');

    // 断言标签页按钮存在（v0.26.40 八 Tab：模型 | Agent | 写作 | 提示词 | 扩展 | 外观 | 关于 | 账号）
    await expect(page.locator('text=模型').first()).toBeVisible();
    await expect(page.locator('text=Agent').first()).toBeVisible();
    await expect(page.locator('text=写作').first()).toBeVisible();
    await expect(page.locator('text=提示词').first()).toBeVisible();
    await expect(page.locator('text=扩展').first()).toBeVisible();
    await expect(page.locator('text=外观').first()).toBeVisible();
    await expect(page.locator('text=关于').first()).toBeVisible();
    await expect(page.locator('text=账号').first()).toBeVisible();

    // 默认选中模型标签
    await expect(page.locator('main')).toContainText('模型管理');

    expect(consoleErrors.filter(e => !e.includes('enablePersistence'))).toHaveLength(0);
  });

  test('设置页面可切换标签页', async ({ page }) => {
    await page.addInitScript(getMockTauriInitScript());
    await page.goto('/index.html');

    // 进入设置页面
    await page.locator('nav').locator('text=设置').first().click();
    await page.waitForTimeout(1000);

    // 切换到外观（包含编辑器通用设置）
    await page.locator('text=外观').first().click();
    await page.waitForTimeout(500);
    await expect(page.locator('main')).toContainText('外观');

    // 切换到账号
    await page.locator('text=账号').first().click();
    await page.waitForTimeout(500);
    await expect(page.locator('main')).toContainText('账号');
  });

  test('世界构建页面加载无报错', async ({ page }) => {
    const consoleErrors: string[] = [];
    page.on('console', msg => {
      if (msg.type() === 'error') consoleErrors.push(msg.text());
    });
    page.on('pageerror', err => consoleErrors.push(err.message));

    await page.addInitScript(getMockTauriInitScript());
    await page.goto('/index.html');

    // 点击世界构建导航
    await page.locator('nav').locator('text=世界构建').first().click();
    await page.waitForTimeout(1000);

    await expect(page.locator('main')).toBeVisible();
    expect(consoleErrors.filter(e => !e.includes('enablePersistence'))).toHaveLength(0);
  });

  test('知识图谱页面加载无报错', async ({ page }) => {
    const consoleErrors: string[] = [];
    page.on('console', msg => {
      if (msg.type() === 'error') consoleErrors.push(msg.text());
    });
    page.on('pageerror', err => consoleErrors.push(err.message));

    await page.addInitScript(getMockTauriInitScript());
    await page.goto('/index.html');

    // 点击知识图谱导航
    await page.locator('nav').locator('text=知识图谱').first().click();
    await page.waitForTimeout(1000);

    await expect(page.locator('main')).toBeVisible();
    expect(consoleErrors.filter(e => !e.includes('enablePersistence'))).toHaveLength(0);
  });
});
