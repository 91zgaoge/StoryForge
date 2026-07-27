import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useGhostChrome } from '../useGhostChrome';

describe('useGhostChrome', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('enters ghost mode after idle delay', () => {
    const { result } = renderHook(() => useGhostChrome(true));

    expect(result.current.ghost).toBe(false);

    act(() => vi.advanceTimersByTime(3000));
    expect(result.current.ghost).toBe(true);
  });

  it('resets timer on mousemove', () => {
    const { result } = renderHook(() => useGhostChrome(true));

    act(() => vi.advanceTimersByTime(2500));
    act(() => window.dispatchEvent(new MouseEvent('mousemove')));
    act(() => vi.advanceTimersByTime(500));

    expect(result.current.ghost).toBe(false);

    act(() => vi.advanceTimersByTime(3000));
    expect(result.current.ghost).toBe(true);
  });

  it('resets timer on keydown, click and touchstart', () => {
    const { result } = renderHook(() => useGhostChrome(true));

    act(() => vi.advanceTimersByTime(2500));
    act(() => window.dispatchEvent(new KeyboardEvent('keydown')));
    act(() => vi.advanceTimersByTime(500));
    expect(result.current.ghost).toBe(false);

    act(() => vi.advanceTimersByTime(2500));
    act(() => window.dispatchEvent(new MouseEvent('click')));
    act(() => vi.advanceTimersByTime(500));
    expect(result.current.ghost).toBe(false);

    act(() => vi.advanceTimersByTime(2500));
    act(() => window.dispatchEvent(new TouchEvent('touchstart')));
    act(() => vi.advanceTimersByTime(500));
    expect(result.current.ghost).toBe(false);

    act(() => vi.advanceTimersByTime(3000));
    expect(result.current.ghost).toBe(true);
  });

  it('stays visible when disabled', () => {
    const { result } = renderHook(() => useGhostChrome(false));

    expect(result.current.ghost).toBe(false);

    act(() => vi.advanceTimersByTime(3000));
    expect(result.current.ghost).toBe(false);
  });

  it('hideChrome immediately enters ghost mode and clears pending timer', () => {
    const { result } = renderHook(() => useGhostChrome(true));

    act(() => vi.advanceTimersByTime(1000));
    expect(result.current.ghost).toBe(false);

    act(() => result.current.hideChrome());
    expect(result.current.ghost).toBe(true);

    act(() => vi.advanceTimersByTime(3000));
    expect(result.current.ghost).toBe(true);
  });

  it('showChrome manually resets the ghost timer', () => {
    const { result } = renderHook(() => useGhostChrome(true));

    act(() => vi.advanceTimersByTime(2500));
    act(() => result.current.showChrome());
    act(() => vi.advanceTimersByTime(500));

    expect(result.current.ghost).toBe(false);

    act(() => vi.advanceTimersByTime(3000));
    expect(result.current.ghost).toBe(true);
  });
});
