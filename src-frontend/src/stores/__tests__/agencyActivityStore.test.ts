import { describe, it, expect, beforeEach } from 'vitest';
import { useAgencyActivityStore } from '../agencyActivityStore';

function resetStore() {
  useAgencyActivityStore.setState({ activities: [], progress: [], activeRunId: null });
}

describe('agencyActivityStore', () => {
  beforeEach(resetStore);

  it('appendActivity 追加事件并自动 setActiveRunId', () => {
    useAgencyActivityStore
      .getState()
      .appendActivity({ run_id: 'r1', role: 'lead_writer', action: 'start', detail: '首章' });
    const s = useAgencyActivityStore.getState();
    expect(s.activities).toHaveLength(1);
    expect(s.activities[0].role).toBe('lead_writer');
    expect(typeof s.activities[0].at).toBe('number');
    expect(s.activeRunId).toBe('r1');
  });

  it('appendProgress 追加事件并自动 setActiveRunId', () => {
    useAgencyActivityStore
      .getState()
      .appendProgress({ run_id: 'r2', phase: 'writing', status: 'running', message: '写第一章' });
    const s = useAgencyActivityStore.getState();
    expect(s.progress).toHaveLength(1);
    expect(s.progress[0].phase).toBe('writing');
    expect(typeof s.progress[0].at).toBe('number');
    expect(s.activeRunId).toBe('r2');
  });

  it('activities 超上限 200 时丢弃最旧', () => {
    for (let i = 0; i < 210; i++) {
      useAgencyActivityStore
        .getState()
        .appendActivity({ run_id: 'r1', role: 'producer', action: 'done', detail: `事件${i}` });
    }
    const s = useAgencyActivityStore.getState();
    expect(s.activities).toHaveLength(200);
    // 最旧的 10 条被丢弃，最旧保留的是 事件10
    expect(s.activities[0].detail).toBe('事件10');
    expect(s.activities[199].detail).toBe('事件209');
  });

  it('progress 超上限 200 时丢弃最旧', () => {
    for (let i = 0; i < 210; i++) {
      useAgencyActivityStore
        .getState()
        .appendProgress({ run_id: 'r1', phase: `p${i}`, status: 'running', message: '' });
    }
    const s = useAgencyActivityStore.getState();
    expect(s.progress).toHaveLength(200);
    expect(s.progress[0].phase).toBe('p10');
    expect(s.progress[199].phase).toBe('p209');
  });

  it('hydrateFromRuns：activeRunId 为空时取最新 run', () => {
    useAgencyActivityStore.getState().hydrateFromRuns([{ id: 'r9' }, { id: 'r8' }]);
    expect(useAgencyActivityStore.getState().activeRunId).toBe('r9');
  });

  it('hydrateFromRuns：已有 activeRunId 时不覆盖（实时事件优先）', () => {
    useAgencyActivityStore.getState().setActiveRunId('live-run');
    useAgencyActivityStore.getState().hydrateFromRuns([{ id: 'r9' }]);
    expect(useAgencyActivityStore.getState().activeRunId).toBe('live-run');
  });

  it('hydrateFromRuns：runs 为空时不动', () => {
    useAgencyActivityStore.getState().hydrateFromRuns([]);
    expect(useAgencyActivityStore.getState().activeRunId).toBeNull();
  });

  it('setActiveRunId 直接切换当前 run', () => {
    useAgencyActivityStore.getState().setActiveRunId('r5');
    expect(useAgencyActivityStore.getState().activeRunId).toBe('r5');
  });

  it('多 run 事件并存时可按 activeRunId 过滤出当前 run 的事件', () => {
    const store = useAgencyActivityStore.getState();
    store.appendActivity({ run_id: 'r1', role: 'lead_writer', action: 'start', detail: '首章' });
    store.appendActivity({ run_id: 'r2', role: 'producer', action: 'start', detail: '概念' });
    const s = useAgencyActivityStore.getState();
    // 后到的 run 事件把 activeRunId 切到 r2
    expect(s.activeRunId).toBe('r2');
    const filtered = s.activities.filter(a => a.run_id === s.activeRunId);
    expect(filtered).toHaveLength(1);
    expect(filtered[0].detail).toBe('概念');
  });
});
