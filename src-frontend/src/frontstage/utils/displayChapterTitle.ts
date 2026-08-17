/**
 * 幕前章节标题展示规则。
 * - 空/空白 → 「第N章」
 * - 「第2章」「第一章」这类由章号派生的标题 → 跟当前 chapter_number
 *   （自动分章重排后库里的标题会落后，不能照显示）
 * - 否则 → trim 后的真实标题
 */
const GENERIC_CHAPTER_TITLE = /^第[0-9一二三四五六七八九十百千零〇两]+章$/;

export function displayChapterTitle(
  chapter: { title?: string | null; chapter_number: number } | null | undefined
): string {
  if (!chapter) return '';
  const t = (chapter.title ?? '').trim();
  if (!t || GENERIC_CHAPTER_TITLE.test(t)) {
    return `第${chapter.chapter_number}章`;
  }
  return t;
}
