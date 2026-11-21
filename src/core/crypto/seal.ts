import { randomBytes, utf8 } from './encoding';

/** Version of the sealed-record format (not of the payload schema). */
export const SEAL_VERSION = 1;

export interface Sealed {
  v: number;
  iv: Uint8Array<ArrayBuffer>;
  ct: Uint8Array<ArrayBuffer>;
}

/** Associated data binds a ciphertext to its record id: records cannot be swapped. */
function aad(id: string, v: number): Uint8Array<ArrayBuffer> {
  return utf8(`kuracare:${v}:${id}`);
}

export async function seal(dek: CryptoKey, id: string, value: unknown): Promise<Sealed> {
  const iv = randomBytes(12);
  const plain = utf8(JSON.stringify(value));
  const ct = await crypto.subtle.encrypt({ name: 'AES-GCM', iv, additionalData: aad(id, SEAL_VERSION) }, dek, plain);
  return { v: SEAL_VERSION, iv, ct: new Uint8Array(ct) };
}

export async function unseal<T = unknown>(dek: CryptoKey, id: string, sealed: Sealed): Promise<T> {
  if (sealed.v !== SEAL_VERSION) throw new Error(`Unsupported seal version ${sealed.v}`);
  const plain = await crypto.subtle.decrypt(
    { name: 'AES-GCM', iv: sealed.iv, additionalData: aad(id, sealed.v) },
    dek,
    sealed.ct,
  );
  return JSON.parse(new TextDecoder().decode(plain)) as T;
}
