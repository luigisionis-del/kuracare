import { deriveKek, newKdfParams, DEFAULT_KDF, type KdfParams } from '../crypto/kdf';
import { generateWrappedDek, rewrapDek, unwrapDek } from '../crypto/keyring';
import { seal, unseal } from '../crypto/seal';
import { KuraDb } from './db';

export class VaultLockedError extends Error {
  constructor() {
    super('Vault is locked');
    this.name = 'VaultLockedError';
  }
}

/**
 * Encrypted key-value store on top of IndexedDB.
 * Nothing can be written before a passphrase exists (create) or while locked.
 */
export class Vault {
  #dek: CryptoKey | null = null;

  constructor(
    readonly db: KuraDb = new KuraDb(),
    private readonly kdfBase: Omit<KdfParams, 'salt'> = DEFAULT_KDF,
  ) {}

  async exists(): Promise<boolean> {
    return (await this.db.vault.get('main')) !== undefined;
  }

  get unlocked(): boolean {
    return this.#dek !== null;
  }

  async create(passphrase: string): Promise<void> {
    if (await this.exists()) throw new Error('Vault already exists');
    const kdf = newKdfParams(this.kdfBase);
    const kek = await deriveKek(passphrase, kdf);
    const { wrapped, dek } = await generateWrappedDek(kek);
    await this.db.vault.put({ id: 'main', format: 1, kdf, wrappedDek: wrapped });
    this.#dek = dek;
  }

  /** Throws WrongPassphraseError on a wrong passphrase. */
  async unlock(passphrase: string): Promise<void> {
    const header = await this.db.vault.get('main');
    if (!header) throw new Error('No vault');
    const kek = await deriveKek(passphrase, header.kdf);
    this.#dek = await unwrapDek(header.wrappedDek, kek);
  }

  lock(): void {
    this.#dek = null;
  }

  async changePassphrase(oldPassphrase: string, newPassphrase: string): Promise<void> {
    const header = await this.db.vault.get('main');
    if (!header) throw new Error('No vault');
    const oldKek = await deriveKek(oldPassphrase, header.kdf);
    const kdf = newKdfParams(this.kdfBase);
    const newKek = await deriveKek(newPassphrase, kdf);
    const wrappedDek = await rewrapDek(header.wrappedDek, oldKek, newKek);
    await this.db.vault.put({ ...header, kdf, wrappedDek });
  }

  #key(): CryptoKey {
    if (!this.#dek) throw new VaultLockedError();
    return this.#dek;
  }

  async put(id: string, value: unknown): Promise<void> {
    const sealed = await seal(this.#key(), id, value);
    await this.db.records.put({ id, ...sealed });
  }

  async get<T>(id: string): Promise<T | undefined> {
    const key = this.#key();
    const row = await this.db.records.get(id);
    return row ? unseal<T>(key, id, row) : undefined;
  }

  async remove(id: string): Promise<void> {
    this.#key();
    await this.db.records.delete(id);
  }

  /** Decrypts every record whose id starts with `prefix`. Volumes are small (thousands). */
  async list<T>(prefix: string): Promise<T[]> {
    const key = this.#key();
    const rows = await this.db.records.where('id').startsWith(prefix).toArray();
    return Promise.all(rows.map((r) => unseal<T>(key, r.id, r)));
  }

  /** Irreversible: deletes the whole database, header included. */
  async destroy(): Promise<void> {
    this.lock();
    this.db.close();
    await this.db.delete();
  }
}
