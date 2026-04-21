import { test, expect } from '@playwright/test';

/**
 * 模型设置 E2E 测试
 * 覆盖设置页面加载和基础 UI 结构
 */
test.describe('模型设置', () => {
  test.beforeEach(async ({ page }) => {
    await page.setViewportSize({ width: 1920, height: 1080 });
  });

  test('设置页面加载并显示标签页', async ({ page }) => {
    await page.goto('/index.html#/settings');
    await page.waitForTimeout(2000);

    // 验证头部存在
    await expect(page.locator('text=工作室配置')).toBeVisible();

    // 验证标签页导航存在
    await expect(page.locator('text=聊天模型')).toBeVisible();
    await expect(page.locator('text=嵌入模型')).toBeVisible();
    await expect(page.locator('text=多模态')).toBeVisible();
    await expect(page.locator('text=图像生成')).toBeVisible();
    await expect(page.locator('text=Agent配置')).toBeVisible();
    await expect(page.locator('text=通用设置')).toBeVisible();

    // 截图记录
    await page.screenshot({
      path: 'e2e/screenshots/settings_loaded.png',
      fullPage: true,
    });
  });

  test('切换标签页正常工作', async ({ page }) => {
    await page.goto('/index.html#/settings');
    await page.waitForTimeout(2000);

    // 点击嵌入模型标签
    await page.click('text=嵌入模型');
    await page.waitForTimeout(500);

    // 验证嵌入模型标签被激活（通过样式判断）
    await expect(page.locator('text=嵌入模型配置').first()).toBeVisible();

    // 点击多模态标签
    await page.click('text=多模态');
    await page.waitForTimeout(500);

    await expect(page.locator('text=多模态模型配置').first()).toBeVisible();
  });

  test('通用设置页面显示版本信息', async ({ page }) => {
    await page.goto('/index.html#/settings');
    await page.waitForTimeout(2000);

    // 点击通用设置
    await page.click('text=通用设置');
    await page.waitForTimeout(500);

    // 验证版本信息
    await expect(page.locator('text=StoryForge (草苔)')).toBeVisible();
  });
});
