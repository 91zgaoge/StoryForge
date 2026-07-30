import { describe, expect, it } from 'vitest';

import { extractMessage, parseStructuredError } from '../errorHandler';

/**
 * 回归测试：issue #12 -- 创作生成失败时 toast 显示 "[object Object]"
 *
 * 根因：后端 AppError 序列化为普通对象 { code, message, severity }，
 * Tauri v2.4 将其作为普通对象（非 Error 实例）投递到前端 catch 块。
 * 旧代码用 `String(err)` 或 `err instanceof Error ? err.message : String(err)`
 * 转字符串，对普通对象产出 "[object Object]"，可读的 message 被丢弃。
 * v0.30.37 统一改用 extractMessage 提取 message。
 */
describe('extractMessage - issue #12 回归', () => {
  it('从 AppError 普通对象提取 message（不产出 [object Object]）', () => {
    const appError = {
      code: 'LLM_GENERATION_TIMEOUT',
      message: '模型响应超时，请检查模型服务',
      severity: 'Retry' as const,
    };
    expect(extractMessage(appError)).toBe('模型响应超时，请检查模型服务');
    // 关键断言：绝不产出 "[object Object]"
    expect(extractMessage(appError)).not.toBe('[object Object]');
  });

  it('从带 data 的 AppError 普通对象提取 message', () => {
    const appError = {
      code: 'PREFLIGHT_FAILED',
      message: '写作前检查未通过',
      severity: 'UserAction' as const,
      data: { issues: ['缺少世界观合同'] },
    };
    expect(extractMessage(appError)).toBe('写作前检查未通过');
  });

  it('parseStructuredError 识别 AppError 普通对象', () => {
    const appError = { code: 'DB_LOCKED', message: '数据库繁忙', severity: 'Retry' };
    const structured = parseStructuredError(appError);
    expect(structured).not.toBeNull();
    expect(structured?.code).toBe('DB_LOCKED');
    expect(structured?.message).toBe('数据库繁忙');
    expect(structured?.severity).toBe('Retry');
  });

  it('从 Error.message 内嵌 JSON 的旧式 Tauri 投递提取 message', () => {
    const err = new Error(JSON.stringify({ code: 'CANCELLATION', message: '操作已取消' }));
    expect(extractMessage(err)).toBe('操作已取消');
  });

  it('普通 Error 实例返回 message', () => {
    expect(extractMessage(new Error('网络异常'))).toBe('网络异常');
  });

  it('字符串直接返回', () => {
    expect(extractMessage('简单错误文本')).toBe('简单错误文本');
  });

  it('带 message 字段的普通对象返回 message', () => {
    const obj = { message: '自定义消息', foo: 1 };
    expect(extractMessage(obj)).toBe('自定义消息');
  });

  it('完全无法识别的值返回兜底文案（非 [object Object]）', () => {
    expect(extractMessage(undefined)).toBe('Unknown error');
    expect(extractMessage(null)).toBe('Unknown error');
    expect(extractMessage(42)).toBe('Unknown error');
    expect(extractMessage({ code: 123 })).toBe('Unknown error');
  });
});
