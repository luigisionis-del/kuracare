import type { Vault } from './store/vault';

export interface TimeWindow {
  /** "HH:MM", local time */
  start: string;
  end: string;
}

export interface Settings {
  version: 1;
  checkinWindows: TimeWindow[];
  lockAfterMinutes: number;
  /** ISO date; while in the future, the app asks nothing. */
  pauseUntil: string | null;
}

export const SETTINGS_ID = 'cfg:settings';

export const DEFAULT_SETTINGS: Settings = {
  version: 1,
  checkinWindows: [],
  lockAfterMinutes: 5,
  pauseUntil: null,
};

export async function loadSettings(vault: Vault): Promise<Settings> {
  return { ...DEFAULT_SETTINGS, ...((await vault.get<Settings>(SETTINGS_ID)) ?? {}) };
}

export async function saveSettings(vault: Vault, settings: Settings): Promise<void> {
  await vault.put(SETTINGS_ID, settings);
}

const HHMM = /^([01]\d|2[0-3]):[0-5]\d$/;

export function isValidWindow(w: TimeWindow): boolean {
  return HHMM.test(w.start) && HHMM.test(w.end) && w.start < w.end;
}
