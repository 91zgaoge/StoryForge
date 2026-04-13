import { test, expect } from '@playwright/test';

/**
 * 示例测试 - 演示 Playwright 基本功能
 */
test.describe('StoryForge 基本测试', () => {
  
  test('首页加载测试', async ({ page }) => {
    // 导航到首页
    await page.goto('/');
    
    // 等待页面加载
    await page.waitForLoadState('networkidle');
    
    // 截图保存
    await page.screenshot({ path: 'e2e/screenshots/homepage.png', fullPage: true });
    
    // 验证页面标题
    const title = await page.title();
    console.log('页面标题:', title);
    
    // 验证 body 元素存在
    await expect(page.locator('body')).toBeVisible();
  });

  test('幕前界面测试', async ({ page }) => {
    // 导航到幕前界面
    await page.goto('/frontstage.html');
    
    // 等待加载
    await page.waitForTimeout(2000);
    
    // 截图
    await page.screenshot({ path: 'e2e/screenshots/frontstage.png', fullPage: true });
    
    console.log('幕前界面截图已保存');
  });

  test('幕后界面测试', async ({ page }) => {
    // 导航到幕后界面
    await page.goto('/index.html');
    
    // 等待加载
    await page.waitForTimeout(2000);
    
    // 截图
    await page.screenshot({ path: 'e2e/screenshots/backstage.png', fullPage: true });
    
    console.log('幕后界面截图已保存');
  });
});
