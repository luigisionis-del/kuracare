/**
 * Calls `onIdle` after `minutes` without user activity, also when the app
 * comes back from the background after that long. Returns a stop function.
 */
export function startIdleLock(minutes: number, onIdle: () => void): () => void {
  const limit = minutes * 60_000;
  let last = Date.now();
  const touch = () => (last = Date.now());
  const check = () => {
    if (Date.now() - last >= limit) onIdle();
  };
  const onVisibility = () => {
    if (document.visibilityState === 'visible') check();
  };
  const events = ['pointerdown', 'keydown', 'scroll', 'touchstart'] as const;
  for (const e of events) window.addEventListener(e, touch, { passive: true });
  document.addEventListener('visibilitychange', onVisibility);
  const timer = window.setInterval(check, 15_000);
  return () => {
    for (const e of events) window.removeEventListener(e, touch);
    document.removeEventListener('visibilitychange', onVisibility);
    window.clearInterval(timer);
  };
}
