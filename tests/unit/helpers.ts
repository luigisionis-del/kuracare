import { KuraDb } from '../../src/core/store/db';
import { Vault } from '../../src/core/store/vault';

/** Light Argon2 parameters: tests check behaviour, not KDF cost. */
export const FAST_KDF = { alg: 'argon2id', memoryKiB: 256, iterations: 1, parallelism: 1 } as const;

let n = 0;
export function freshVault(): Vault {
  return new Vault(new KuraDb(`test-${Date.now()}-${n++}`), FAST_KDF);
}
