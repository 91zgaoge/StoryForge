import { describe, it, expect } from 'vitest';
import { friendlyText } from '../useAgencyAgentActivity';

describe('friendlyText（DETAIL_VERB 映射）', () => {
  it('静态 detail 进行中：映射为动词短语', () => {
    expect(friendlyText('producer', 'start', '概念')).toBe('管理正在构思概念');
    expect(friendlyText('lead_writer', 'start', '首章')).toBe('主创正在写第一章');
    expect(friendlyText('producer', 'start', '深度资产')).toBe('管理正在生成深度资产');
    expect(friendlyText('producer', 'start', '装配')).toBe('管理正在装配最终稿');
    expect(friendlyText('producer', 'start', '资产')).toBe('管理正在生成资产');
    expect(friendlyText('producer', 'start', '资产补齐')).toBe('管理正在补齐资产');
    expect(friendlyText('editor_auditor', 'start', '后台审查')).toBe('编辑审计正在后台质检');
  });

  it('带章节号的动态 detail 进行中：按模式生成动词短语', () => {
    expect(friendlyText('lead_writer', 'start', '第3章')).toBe('主创正在写第3章');
    expect(friendlyText('lead_writer', 'start', '第3章草稿')).toBe('主创正在写第3章草稿');
    expect(friendlyText('editor_auditor', 'start', '审查第3章')).toBe('编辑审计正在质检第3章');
  });

  it('已完成：直接用 detail 作宾语，不走动词映射', () => {
    expect(friendlyText('producer', 'done', '概念')).toBe('管理已完成概念');
    expect(friendlyText('producer', 'done', '资产补齐')).toBe('管理已完成资产补齐');
    expect(friendlyText('lead_writer', 'done', '第3章草稿')).toBe('主创已完成第3章草稿');
    expect(friendlyText('editor_auditor', 'done', '审查第3章')).toBe('编辑审计已完成审查第3章');
  });

  it('未命中映射的 detail 回退为原文', () => {
    expect(friendlyText('lead_writer', 'start', '未知动作')).toBe('主创正在未知动作');
  });

  it('未知角色回退为 role 原值', () => {
    expect(friendlyText('ghost', 'start', '概念')).toBe('ghost正在构思概念');
  });
});
