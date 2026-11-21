import { argon2id } from 'hash-wasm';
import { fromBase64, randomBytes, toBase64 } from './encoding';

/**
 * Passphrase -> key-encryption key (KEK) via Argon2id.
 *
 * Argon2id is memory-hard: brute-forcing a passphrase taken from a stolen
 * device costs far more on GPUs than with PBKDF2. Parameters are stored next
 * to the wrapped key so they can be raised later without breaking old vaults.
 */
export interface KdfParams {
  alg: 'argon2id';
  memoryKiB: number;
  iterations: number;
  parallelism: number;
  /** base64, 16 random bytes */
  salt: string;
}

/** ~64 MiB, 3 passes: well above the OWASP minimum, under ~1 s on a recent phone. */
export const DEFAULT_KDF: Omit<KdfParams, 'salt'> = {
  alg: 'argon2id',
  memoryKiB: 64 * 1024,
  iterations: 3,
  parallelism: 1,
};

export function newKdfParams(base: Omit<KdfParams, 'salt'> = DEFAULT_KDF): KdfParams {
  return { ...base, salt: toBase64(randomBytes(16)) };
}

/** Normalised so that the same passphrase typed on different keyboards matches. */
export function normalisePassphrase(passphrase: string): string {
  return passphrase.normalize('NFC');
}

export async function deriveKek(passphrase: string, params: KdfParams): Promise<CryptoKey> {
  if (params.alg !== 'argon2id') throw new Error(`Unsupported KDF: ${params.alg}`);
  const raw = (await argon2id({
    password: normalisePassphrase(passphrase),
    salt: fromBase64(params.salt),
    parallelism: params.parallelism,
    iterations: params.iterations,
    memorySize: params.memoryKiB,
    hashLength: 32,
    outputType: 'binary',
  })) as Uint8Array<ArrayBuffer>;
  try {
    return await crypto.subtle.importKey('raw', raw, 'AES-KW', false, ['wrapKey', 'unwrapKey']);
  } finally {
    raw.fill(0);
  }
}
