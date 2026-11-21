import { Vault } from '../core/store/vault';
import { DEFAULT_SETTINGS, loadSettings, saveSettings, type Settings } from '../core/settings';
import { startIdleLock } from './idle';

export type Phase = 'loading' | 'onboarding' | 'locked' | 'unlocked';

export const vault = new Vault();

export const session = $state<{ phase: Phase; settings: Settings }>({
  phase: 'loading',
  settings: { ...DEFAULT_SETTINGS },
});

let stopIdle: (() => void) | null = null;

export async function boot(): Promise<void> {
  session.phase = (await vault.exists()) ? 'locked' : 'onboarding';
}

async function afterUnlock(): Promise<void> {
  session.settings = await loadSettings(vault);
  session.phase = 'unlocked';
  stopIdle?.();
  stopIdle = startIdleLock(session.settings.lockAfterMinutes, lock);
  // Ask the browser not to evict our data under storage pressure.
  try {
    await navigator.storage?.persist?.();
  } catch {
    /* not available: backups remain the safety net */
  }
}

export async function createVault(passphrase: string, settings: Settings): Promise<void> {
  await vault.create(passphrase);
  await saveSettings(vault, settings);
  await afterUnlock();
}

export async function unlock(passphrase: string): Promise<void> {
  await vault.unlock(passphrase);
  await afterUnlock();
}

export function lock(): void {
  stopIdle?.();
  stopIdle = null;
  vault.lock();
  session.settings = { ...DEFAULT_SETTINGS };
  session.phase = 'locked';
}

export async function destroyEverything(): Promise<void> {
  stopIdle?.();
  await vault.destroy();
  location.reload();
}
