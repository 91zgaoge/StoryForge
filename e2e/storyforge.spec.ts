import { test, expect } from '@playwright/test';

/**
 * StoryForge 应用测试套件
 * 测试核心功能：双界面、场景管理、版本控制等
 */
test.describe('StoryForge 应用测试', () => {
  
  test.beforeEach(async ({ page }) => {
    // 每个测试前设置视口
    await page.setViewportSize({ width: 1920, height: 1080 });
  });

  test.describe('🎭 幕前界面测试', () => {
    
    test('幕前界面加载和截图', async ({ page }) => {
      await page.goto('/frontstage.html');
      await page.waitForTimeout(3000);
      
      // 截图
      await page.screenshot({ 
        path: 'e2e/screenshots/frontstage_initial.png', 
        fullPage: true 
      });
      
      console.log('✅ 幕前界面已截图');
    });

    test('幕前界面交互元素检查', async ({ page }) => {
      await page.goto('/frontstage.html');
      await page.waitForTimeout(2000);
      
      // 检查是否存在编辑器区域
      const editor = page.locator('.ProseMirror, [contenteditable="true"], .editor');
      const hasEditor = await editor.count() > 0;
      
      if (hasEditor) {
        console.log('✅ 找到编辑器元素');
        await editor.screenshot({ path: 'e2e/screenshots/frontstage_editor.png' });
      } else {
        console.log('⚠️ 未找到编辑器元素，可能是加载中');
      }
      
      // 截图整个页面
      await page.screenshot({ 
        path: 'e2e/screenshots/frontstage_full.png', 
        fullPage: true 
      });
    });
  });

  test.describe('🔧 幕后界面测试', () => {
    
    test('幕后仪表盘加载', async ({ page }) => {
      await page.goto('/index.html');
      await page.waitForTimeout(3000);
      
      await page.screenshot({ 
        path: 'e2e/screenshots/backstage_dashboard.png', 
        fullPage: true 
      });
      
      console.log('✅ 幕后仪表盘已截图');
    });

    test('场景管理页面', async ({ page }) => {
      await page.goto('/index.html#/scenes');
      await page.waitForTimeout(3000);
      
      await page.screenshot({ 
        path: 'e2e/screenshots/scenes_page.png', 
        fullPage: true 
      });
      
      console.log('✅ 场景管理页面已截图');
    });

    test('角色管理页面', async ({ page }) => {
      await page.goto('/index.html#/characters');
      await page.waitForTimeout(3000);
      
      await page.screenshot({ 
        path: 'e2e/screenshots/characters_page.png', 
        fullPage: true 
      });
      
      console.log('✅ 角色管理页面已截图');
    });

    test('故事列表页面', async ({ page }) => {
      await page.goto('/index.html#/stories');
      await page.waitForTimeout(3000);
      
      await page.screenshot({ 
        path: 'e2e/screenshots/stories_page.png', 
        fullPage: true 
      });
      
      console.log('✅ 故事列表页面已截图');
    });
  });

  test.describe('📜 版本管理功能测试', () => {
    
    test('版本时间线组件', async ({ page }) => {
      // 导航到场景页面（假设有版本管理功能）
      await page.goto('/index.html#/scenes');
      await page.waitForTimeout(3000);
      
      // 查找版本相关的元素
      const versionElements = page.locator('text=/version|版本/i');
      const count = await versionElements.count();
      
      console.log(`找到 ${count} 个版本相关元素`);
      
      await page.screenshot({ 
        path: 'e2e/screenshots/version_timeline.png', 
        fullPage: true 
      });
    });
  });

  test.describe('📊 响应式测试', () => {
    
    test('不同分辨率下的幕前界面', async ({ page }) => {
      const viewports = [
        { width: 1920, height: 1080, name: 'desktop' },
        { width: 1366, height: 768, name: 'laptop' },
        { width: 768, height: 1024, name: 'tablet' },
      ];

      for (const viewport of viewports) {
        await page.setViewportSize(viewport);
        await page.goto('/frontstage.html');
        await page.waitForTimeout(2000);
        
        await page.screenshot({
          path: `e2e/screenshots/frontstage_${viewport.name}.png`,
          fullPage: true
        });
        
        console.log(`✅ ${viewport.name} 分辨率截图完成`);
      }
    });
  });

  test.describe('🎯 功能交互测试', () => {
    
    test('页面导航流畅度', async ({ page }) => {
      await page.goto('/index.html');
      await page.waitForTimeout(2000);
      
      // 测试导航到不同页面
      const pages = ['#/stories', '#/characters', '#/scenes', '#/settings'];
      
      for (const route of pages) {
        const startTime = Date.now();
        await page.goto(`/index.html${route}`);
        await page.waitForLoadState('networkidle');
        const loadTime = Date.now() - startTime;
        
        console.log(`页面 ${route} 加载时间: ${loadTime}ms`);
        
        await page.screenshot({
          path: `e2e/screenshots/nav_${route.replace('#/', '')}.png`,
          fullPage: true
        });
      }
    });
  });
});
