/**
 * Two-key scheme:
 *  - DEK (data-encryption key): random AES-GCM-256, encrypts every record.
 *  - KEK (key-encryption key): derived from the passphrase, wraps the DEK (AES-KW).
 * Changing the passphrase only re-wraps the DEK; records are untouched.
 * The DEK kept in memory while unlocked is non-extractable.
 */

export class WrongPassphraseError extends Error {
  constructor() {
    super('Wrong passphrase');
    this.name = 'WrongPassphraseError';
  }
}

const DEK_ALG: AesKeyGenParams = { name: 'AES-GCM', length: 256 };

export async function generateWrappedDek(kek: CryptoKey): Promise<{ wrapped: Uint8Array<ArrayBuffer>; dek: CryptoKey }> {
  const extractable = await crypto.subtle.generateKey(DEK_ALG, true, ['encrypt', 'decrypt']);
  const wrapped = new Uint8Array(await crypto.subtle.wrapKey('raw', extractable, kek, 'AES-KW'));
  // Re-import as non-extractable so the in-memory key can never be exported.
  const dek = await unwrapDek(wrapped, kek, false);
  return { wrapped, dek };
}

export async function unwrapDek(wrapped: Uint8Array<ArrayBuffer>, kek: CryptoKey, extractable = false): Promise<CryptoKey> {
  try {
    return await crypto.subtle.unwrapKey('raw', wrapped, kek, 'AES-KW', DEK_ALG, extractable, ['encrypt', 'decrypt']);
  } catch {
    // AES-KW has a built-in integrity check: a wrong KEK fails here.
    throw new WrongPassphraseError();
  }
}

export async function rewrapDek(wrapped: Uint8Array<ArrayBuffer>, oldKek: CryptoKey, newKek: CryptoKey): Promise<Uint8Array<ArrayBuffer>> {
  const dek = await unwrapDek(wrapped, oldKek, true);
  return new Uint8Array(await crypto.subtle.wrapKey('raw', dek, newKek, 'AES-KW'));
}
