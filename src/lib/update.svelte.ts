import { registerSW } from 'virtual:pwa-register';

/** A new version has been downloaded and waits for an explicit "Aggiorna". */
export const update = $state({ ready: false });

let apply: ((reload?: boolean) => Promise<void>) | null = null;

export function startUpdates(): void {
  if (!('serviceWorker' in navigator)) return;
  apply = registerSW({
    immediate: true,
    onNeedRefresh() {
      update.ready = true;
    },
    // Check for a new version whenever the app is opened from the background.
    onRegisteredSW(_url, reg) {
      if (!reg) return;
      document.addEventListener('visibilitychange', () => {
        if (document.visibilityState === 'visible') reg.update().catch(() => {});
      });
    },
  });
}

export async function applyUpdate(): Promise<void> {
  await apply?.(true);
}
