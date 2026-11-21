import { describe, expect, it } from 'vitest';
import { deriveKek, newKdfParams, DEFAULT_KDF } from '../../src/core/crypto/kdf';
import { generateWrappedDek, unwrapDek, rewrapDek, WrongPassphraseError } from '../../src/core/crypto/keyring';
import { seal, unseal } from '../../src/core/crypto/seal';
import { FAST_KDF } from './helpers';

describe('kdf', () => {
  it('default parameters stay strong', () => {
    expect(DEFAULT_KDF.alg).toBe('argon2id');
    expect(DEFAULT_KDF.memoryKiB).toBeGreaterThanOrEqual(19 * 1024);
    expect(DEFAULT_KDF.iterations).toBeGreaterThanOrEqual(2);
  });

  it('uses a fresh random salt each time', () => {
    expect(newKdfParams(FAST_KDF).salt).not.toBe(newKdfParams(FAST_KDF).salt);
  });

  it('derives the same KEK for NFC/NFD forms of the same passphrase', async () => {
    const p = newKdfParams(FAST_KDF);
    const { wrapped } = await generateWrappedDek(await deriveKek('caffè lungo', p));
    const nfd = 'caffè lungo'.normalize('NFD');
    await expect(unwrapDek(wrapped, await deriveKek(nfd, p))).resolves.toBeDefined();
  });
});

describe('keyring', () => {
  it('rejects a wrong passphrase', async () => {
    const p = newKdfParams(FAST_KDF);
    const { wrapped } = await generateWrappedDek(await deriveKek('giusta', p));
    await expect(unwrapDek(wrapped, await deriveKek('sbagliata', p))).rejects.toBeInstanceOf(WrongPassphraseError);
  });

  it('keeps the in-memory DEK non-extractable', async () => {
    const { dek } = await generateWrappedDek(await deriveKek('x', newKdfParams(FAST_KDF)));
    expect(dek.extractable).toBe(false);
    await expect(crypto.subtle.exportKey('raw', dek)).rejects.toBeDefined();
  });

  it('re-wrapping keeps the same data key', async () => {
    const p1 = newKdfParams(FAST_KDF);
    const p2 = newKdfParams(FAST_KDF);
    const kek1 = await deriveKek('vecchia', p1);
    const { wrapped, dek } = await generateWrappedDek(kek1);
    const sealed = await seal(dek, 'evt:1', { a: 1 });
    const rewrapped = await rewrapDek(wrapped, kek1, await deriveKek('nuova', p2));
    const dek2 = await unwrapDek(rewrapped, await deriveKek('nuova', p2));
    expect(await unseal(dek2, 'evt:1', sealed)).toEqual({ a: 1 });
  });
});

describe('seal', () => {
  it('round-trips JSON and never stores plaintext', async () => {
    const { dek } = await generateWrappedDek(await deriveKek('x', newKdfParams(FAST_KDF)));
    const value = { umore: 7, nota: 'una passeggiata al mare' };
    const sealed = await seal(dek, 'evt:1', value);
    expect(new TextDecoder().decode(sealed.ct)).not.toContain('passeggiata');
    expect(await unseal(dek, 'evt:1', sealed)).toEqual(value);
  });

  it('uses a different IV every time', async () => {
    const { dek } = await generateWrappedDek(await deriveKek('x', newKdfParams(FAST_KDF)));
    const a = await seal(dek, 'evt:1', 1);
    const b = await seal(dek, 'evt:1', 1);
    expect(a.iv).not.toEqual(b.iv);
  });

  it('refuses a ciphertext moved to another record id', async () => {
    const { dek } = await generateWrappedDek(await deriveKek('x', newKdfParams(FAST_KDF)));
    const sealed = await seal(dek, 'evt:1', 'segreto');
    await expect(unseal(dek, 'evt:2', sealed)).rejects.toBeDefined();
  });

  it('detects tampering', async () => {
    const { dek } = await generateWrappedDek(await deriveKek('x', newKdfParams(FAST_KDF)));
    const sealed = await seal(dek, 'evt:1', 'segreto');
    sealed.ct[0]! ^= 1;
    await expect(unseal(dek, 'evt:1', sealed)).rejects.toBeDefined();
  });
});
