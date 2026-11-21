import Dexie, { type Table } from 'dexie';
import type { KdfParams } from '../crypto/kdf';
import type { Sealed } from '../crypto/seal';

/**
 * Everything personal lives in `records`, sealed (AES-GCM). In clear text
 * there is only the record id and the vault header (KDF parameters and the
 * wrapped key) — no kinds, no timestamps, no values.
 */
export interface VaultHeader {
  id: 'main';
  format: 1;
  kdf: KdfParams;
  wrappedDek: Uint8Array<ArrayBuffer>;
}

export interface SealedRow extends Sealed {
  id: string;
}

export const DB_NAME = 'kuracare';

export class KuraDb extends Dexie {
  vault!: Table<VaultHeader, string>;
  records!: Table<SealedRow, string>;

  constructor(name = DB_NAME) {
    super(name);
    this.version(1).stores({ vault: 'id', records: 'id' });
  }
}
