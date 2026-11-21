import type { KuraEvent } from '../model/event';

/**
 * Payload migrations, per event kind. `steps[n]` turns a version-n payload
 * into version n+1. Steps are pure functions with their own tests.
 * Stored data is never rewritten in place: events are upgraded when read.
 */
export type MigrationStep = (payload: unknown) => unknown;

export interface KindMigrations {
  latest: number;
  steps: Record<number, MigrationStep>;
}

export class MigrationRegistry {
  #kinds = new Map<string, KindMigrations>();

  register(kind: string, latest: number, steps: Record<number, MigrationStep> = {}): this {
    for (let v = 1; v < latest; v++) {
      if (!steps[v]) throw new Error(`Missing migration ${kind} v${v} -> v${v + 1}`);
    }
    this.#kinds.set(kind, { latest, steps });
    return this;
  }

  latest(kind: string): number {
    const k = this.#kinds.get(kind);
    if (!k) throw new Error(`Unknown event kind: ${kind}`);
    return k.latest;
  }

  upgrade<E extends KuraEvent>(event: E): E {
    const k = this.#kinds.get(event.kind);
    if (!k) throw new Error(`Unknown event kind: ${event.kind}`);
    if (event.schemaVersion > k.latest) {
      throw new Error(`${event.kind} v${event.schemaVersion} is newer than this app (v${k.latest})`);
    }
    let payload = event.payload;
    for (let v = event.schemaVersion; v < k.latest; v++) payload = k.steps[v]!(payload);
    return { ...event, payload, schemaVersion: k.latest };
  }
}

/** App-wide registry; event kinds register here as they are added (tappa 3+). */
export const migrations = new MigrationRegistry();
