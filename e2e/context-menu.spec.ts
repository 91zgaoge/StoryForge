import { test, expect } from '@playwright/test';

test('frontstage context menu appears on right click', async ({ page }) => {
  await page.setViewportSize({ width: 1920, height: 1080 });
  await page.goto('/frontstage.html');
  await page.waitForTimeout(3000);

  const editor = page.locator('.rich-text-editor');
  await expect(editor).toBeVisible();

  // Right click in the editor area
  await editor.click({ button: 'right' });
  await page.waitForTimeout(500);

  // Full page screenshot
  await page.screenshot({
    path: 'e2e/screenshots/frontstage_context_menu.png',
    fullPage: true
  });

  // Find the context menu container by looking for the "添加批注" text, then get its container
  const menuItem = page.locator('text=添加批注');
  await expect(menuItem).toBeVisible();

  // Get computed styles of the menu item's closest fixed-position ancestor
  const styles = await menuItem.evaluate((el) => {
    let ancestor: HTMLElement | null = el as HTMLElement;
    while (ancestor && !ancestor.classList.contains('fixed')) {
      ancestor = ancestor.parentElement;
    }
    if (!ancestor) return null;
    const computed = window.getComputedStyle(ancestor);
    return {
      tagName: ancestor.tagName,
      className: ancestor.className,
      position: computed.position,
      backgroundColor: computed.backgroundColor,
      zIndex: computed.zIndex,
      borderRadius: computed.borderRadius,
      boxShadow: computed.boxShadow,
      border: computed.border,
      width: computed.width,
      minWidth: computed.minWidth,
    };
  });

  console.log('Context menu container styles:', JSON.stringify(styles, null, 2));

  // Also get the menu item's own styles
  const itemStyles = await menuItem.evaluate((el) => {
    const computed = window.getComputedStyle(el);
    return {
      color: computed.color,
      display: computed.display,
    };
  });
  console.log('Menu item styles:', JSON.stringify(itemStyles, null, 2));

  // Screenshot the menu item element
  await menuItem.screenshot({ path: 'e2e/screenshots/menu_item.png' });
});
