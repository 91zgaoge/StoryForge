import React from 'react';
import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import {
  AssetRefreshConfirmDialog,
  type AssetRefreshConfirmDraft,
} from '../components/AssetRefreshConfirmDialog';

const baseDraft: AssetRefreshConfirmDraft = {
  instruction: '写后续的故事大纲，同时生成后续的场景大纲',
  storyId: 'story-1',
  sceneId: 'scene-1',
  storyOutline: '韩雪在首尔雨夜对峙李明',
  sceneOutline: '韩雪举枪，李明停在雨里。',
  showStory: true,
  showScene: true,
};

describe('AssetRefreshConfirmDialog', () => {
  it('确认/取消/重写三键都在，编辑后点确认带回改稿', async () => {
    const onChange = vi.fn();
    const onConfirm = vi.fn();
    const onCancel = vi.fn();
    const onRewrite = vi.fn();
    render(
      <AssetRefreshConfirmDialog
        draft={baseDraft}
        onChange={onChange}
        onConfirm={onConfirm}
        onCancel={onCancel}
        onRewrite={onRewrite}
      />
    );

    expect(screen.getByText('确认大纲')).toBeTruthy();
    const story = screen.getByTestId('asset-refresh-story-outline') as HTMLTextAreaElement;
    expect(story.value).toContain('韩雪');
    await userEvent.type(story, '改');
    expect(onChange).toHaveBeenCalled();

    await userEvent.click(screen.getByRole('button', { name: '确认' }));
    expect(onConfirm).toHaveBeenCalledTimes(1);
    await userEvent.click(screen.getByTestId('asset-refresh-cancel'));
    expect(onCancel).toHaveBeenCalled();
    await userEvent.click(screen.getByRole('button', { name: '重写' }));
    expect(onRewrite).toHaveBeenCalledTimes(1);
  });

  it('取消不触发确认', async () => {
    const onConfirm = vi.fn();
    render(
      <AssetRefreshConfirmDialog
        draft={baseDraft}
        onChange={() => {}}
        onConfirm={onConfirm}
        onCancel={() => {}}
        onRewrite={() => {}}
      />
    );
    await userEvent.click(screen.getByTestId('asset-refresh-cancel'));
    expect(onConfirm).not.toHaveBeenCalled();
  });
});
