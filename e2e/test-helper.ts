import { Page, Browser, chromium, firefox, webkit, BrowserContext } from '@playwright/test';
import * as fs from 'fs';
import * as path from 'path';

/**
 * BrowserTestHelper - 无头浏览器测试助手
 * 专为 AI 助手设计的浏览器自动化工具
 */
export class BrowserTestHelper {
  private browser: Browser | null = null;
  private context: BrowserContext | null = null;
  private page: Page | null = null;
  private screenshotsDir: string;

  constructor(screenshotsDir: string = 'e2e/screenshots') {
    this.screenshotsDir = screenshotsDir;
    this.ensureDir(screenshotsDir);
  }

  /**
   * 启动浏览器
   */
  async start(browserType: 'chromium' | 'firefox' | 'webkit' = 'chromium', headless: boolean = false) {
    console.log(`启动 ${browserType} 浏览器...`);
    
    const browserLauncher = { chromium, firefox, webkit }[browserType];
    
    this.browser = await browserLauncher.launch({
      headless,
      args: ['--disable-web-security', '--disable-features=IsolateOrigins'],
    });

    this.context = await this.browser.newContext({
      viewport: { width: 1920, height: 1080 },
      recordVideo: headless ? { dir: 'e2e/videos' } : undefined,
    });

    this.page = await this.context.newPage();
    
    // 设置控制台日志监听
    this.page.on('console', (msg) => {
      console.log(`[浏览器控制台] ${msg.type()}: ${msg.text()}`);
    });

    console.log('浏览器启动成功');
    return this;
  }

  /**
   * 停止浏览器
   */
  async stop() {
    console.log('关闭浏览器...');
    await this.context?.close();
    await this.browser?.close();
    console.log('浏览器已关闭');
  }

  /**
   * 导航到 URL
   */
  async navigate(url: string) {
    if (!this.page) throw new Error('浏览器未启动');
    console.log(`导航到: ${url}`);
    await this.page.goto(url, { waitUntil: 'networkidle' });
    return this;
  }

  /**
   * 截图
   */
  async screenshot(name: string, fullPage: boolean = true): Promise<string> {
    if (!this.page) throw new Error('浏览器未启动');
    
    const filename = `${Date.now()}_${name}.png`;
    const filepath = path.join(this.screenshotsDir, filename);
    
    await this.page.screenshot({ path: filepath, fullPage });
    console.log(`截图已保存: ${filepath}`);
    
    return filepath;
  }

  /**
   * 点击元素
   */
  async click(selector: string) {
    if (!this.page) throw new Error('浏览器未启动');
    console.log(`点击元素: ${selector}`);
    await this.page.click(selector);
    return this;
  }

  /**
   * 点击包含特定文本的元素
   */
  async clickText(text: string) {
    if (!this.page) throw new Error('浏览器未启动');
    console.log(`点击文本: ${text}`);
    await this.page.getByText(text).click();
    return this;
  }

  /**
   * 在输入框中输入文本
   */
  async type(selector: string, text: string) {
    if (!this.page) throw new Error('浏览器未启动');
    console.log(`在 ${selector} 输入: ${text}`);
    await this.page.fill(selector, text);
    return this;
  }

  /**
   * 清除输入框
   */
  async clear(selector: string) {
    if (!this.page) throw new Error('浏览器未启动');
    console.log(`清除输入框: ${selector}`);
    await this.page.fill(selector, '');
    return this;
  }

  /**
   * 按下键盘按键
   */
  async press(key: string) {
    if (!this.page) throw new Error('浏览器未启动');
    console.log(`按下按键: ${key}`);
    await this.page.keyboard.press(key);
    return this;
  }

  /**
   * 滚动页面
   */
  async scroll(deltaX: number, deltaY: number) {
    if (!this.page) throw new Error('浏览器未启动');
    console.log(`滚动: x=${deltaX}, y=${deltaY}`);
    await this.page.mouse.wheel(deltaX, deltaY);
    return this;
  }

  /**
   * 等待元素出现
   */
  async waitFor(selector: string, timeout: number = 10000) {
    if (!this.page) throw new Error('浏览器未启动');
    console.log(`等待元素: ${selector}`);
    await this.page.waitForSelector(selector, { timeout });
    return this;
  }

  /**
   * 等待文本出现
   */
  async waitForText(text: string, timeout: number = 10000) {
    if (!this.page) throw new Error('浏览器未启动');
    console.log(`等待文本: ${text}`);
    await this.page.waitForSelector(`text=${text}`, { timeout });
    return this;
  }

  /**
   * 执行 JavaScript
   */
  async eval<T>(script: string): Promise<T> {
    if (!this.page) throw new Error('浏览器未启动');
    console.log(`执行 JS: ${script.substring(0, 100)}...`);
    return await this.page.evaluate(script);
  }

  /**
   * 获取页面标题
   */
  async getTitle(): Promise<string> {
    if (!this.page) throw new Error('浏览器未启动');
    return await this.page.title();
  }

  /**
   * 获取页面 URL
   */
  async getUrl(): Promise<string> {
    if (!this.page) throw new Error('浏览器未启动');
    return this.page.url();
  }

  /**
   * 获取元素文本
   */
  async getText(selector: string): Promise<string> {
    if (!this.page) throw new Error('浏览器未启动');
    return await this.page.locator(selector).innerText();
  }

  /**
   * 等待指定时间
   */
  async sleep(ms: number) {
    console.log(`等待 ${ms}ms`);
    await new Promise(resolve => setTimeout(resolve, ms));
    return this;
  }

  /**
   * 检查元素是否存在
   */
  async exists(selector: string): Promise<boolean> {
    if (!this.page) throw new Error('浏览器未启动');
    return await this.page.locator(selector).count() > 0;
  }

  /**
   * 获取页面控制台日志
   */
  async getConsoleLogs(): Promise<string[]> {
    // 需要在启动时设置监听，这里返回空数组作为占位
    return [];
  }

  /**
   * 返回原始 page 对象（用于高级操作）
   */
  getPage(): Page {
    if (!this.page) throw new Error('浏览器未启动');
    return this.page;
  }

  private ensureDir(dir: string) {
    if (!fs.existsSync(dir)) {
      fs.mkdirSync(dir, { recursive: true });
    }
  }
}

/**
 * 快速测试函数 - 一键运行测试
 */
export async function runTest(testFn: (helper: BrowserTestHelper) => Promise<void>) {
  const helper = new BrowserTestHelper();
  
  try {
    await helper.start('chromium', false); // 启动有界面模式便于调试
    await testFn(helper);
    console.log('✅ 测试通过');
  } catch (error) {
    console.error('❌ 测试失败:', error);
    // 失败时截图
    await helper.screenshot('error', true);
    throw error;
  } finally {
    await helper.stop();
  }
}

// CLI 支持
if (require.main === module) {
  console.log('BrowserTestHelper 已加载');
  console.log('使用示例:');
  console.log(`
import { BrowserTestHelper, runTest } from './e2e/test-helper';

// 方式 1: 使用 runTest
runTest(async (helper) => {
  await helper.navigate('http://localhost:5173');
  await helper.screenshot('homepage');
  await helper.click('button');
});

// 方式 2: 手动控制
const helper = new BrowserTestHelper();
await helper.start();
await helper.navigate('http://localhost:5173');
await helper.screenshot('test');
await helper.stop();
  `);
}
