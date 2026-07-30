import { describe, it, expect } from 'vitest';
import { isTextDuplicate, normalizeForDuplicateCheck } from '../isTextDuplicate';

describe('isTextDuplicate', () => {
  const story =
    '人类在无尽的宇宙深处，漩渦星系的最外层行星上，一座被粗劲磨炼成为强硬的生存场景的大城市。';

  it('returns false when existing text is empty', () => {
    expect(isTextDuplicate('', story)).toBe(false);
  });

  it('returns false when generated text is empty', () => {
    expect(isTextDuplicate(story, '')).toBe(false);
  });

  it('returns true when existing text equals generated text', () => {
    expect(isTextDuplicate(story, story)).toBe(true);
  });

  it('returns true when existing text contains generated text with different punctuation/whitespace', () => {
    const formatted = `<p>${story}</p>`;
    const ghost = story.replace(/。/g, '。\n');
    expect(isTextDuplicate(formatted, ghost)).toBe(true);
  });

  it('returns true when generated text is a prefix of existing text (>= 30 normalized chars)', () => {
    // v0.30.41: 使用 >= 30 归一化字符的前缀，低于此阈值的短文本不做去重检查
    const prefix = story.slice(0, 40);
    expect(isTextDuplicate(story, prefix)).toBe(true);
  });

  it('returns false for unrelated texts', () => {
    expect(isTextDuplicate('完全不同的内容', story)).toBe(false);
  });

  it('normalization strips HTML tags', () => {
    const html = '<p>hello <strong>world</strong></p>';
    expect(normalizeForDuplicateCheck(html)).toBe('helloworld');
  });

  // v0.30.41: 最小长度守卫测试
  it('returns false for short generated text (< 30 normalized chars) even if it appears in existing', () => {
    // "续写" 是常见中文词，几乎一定出现在长篇正文中
    // 但 2 个归一化字符不应触发去重，否则打字机首帧被静默丢弃
    const longNovel =
      '这是一部关于黑暗与光明的长篇小说。续写一段新的冒险故事。' + '正文内容'.repeat(100);
    expect(isTextDuplicate(longNovel, '续写')).toBe(false);
    expect(isTextDuplicate(longNovel, '续写\n黑暗。')).toBe(false);
    expect(isTextDuplicate(longNovel, story.slice(0, 20))).toBe(false);
  });

  it('returns true for long generated text that is a real duplicate (>= 30 normalized chars)', () => {
    const longNovel = story + '这是续写的内容，继续讲述故事的发展。';
    const duplicate = story.slice(0, 40);
    expect(isTextDuplicate(longNovel, duplicate)).toBe(true);
  });
});
