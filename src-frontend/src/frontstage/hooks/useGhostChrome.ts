import { useState, useEffect, useCallback, useRef } from 'react';

const GHOST_DELAY_MS = 3000;

export function useGhostChrome(enabled = true) {
  const [ghost, setGhost] = useState(false);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const showChrome = useCallback(() => {
    if (!enabled) return;
    setGhost(false);
    if (timerRef.current) clearTimeout(timerRef.current);
    timerRef.current = setTimeout(() => {
      setGhost(true);
    }, GHOST_DELAY_MS);
  }, [enabled]);

  useEffect(() => {
    if (!enabled) {
      setGhost(false);
      return;
    }

    showChrome();

    const events = ['mousemove', 'keydown', 'click', 'touchstart'] as const;
    const onActivity = () => showChrome();
    events.forEach(e => window.addEventListener(e, onActivity));

    return () => {
      events.forEach(e => window.removeEventListener(e, onActivity));
      if (timerRef.current) clearTimeout(timerRef.current);
    };
  }, [enabled, showChrome]);

  return { ghost, showChrome };
}
