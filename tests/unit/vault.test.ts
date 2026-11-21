import { describe, expect, it } from 'vitest';
import { VaultLockedError } from '../../src/core/store/vault';
import { WrongPassphraseError } from '../../src/core/crypto/keyring';
import { freshVault } from './helpers';

describe('vault', () => {
  it('cannot save anything before a passphrase exists', async () => {
    const v = freshVault();
    expect(await v.exists()).toBe(false);
    await expect(v.put('evt:1', { a: 1 })).rejects.toBeInstanceOf(VaultLockedError);
    expect(await v.db.records.count()).toBe(0);
  });

  it('stores only id + ciphertext in IndexedDB', async () => {
    const v = freshVault();
    await v.create('una frase lunga e mia');
    await v.put('evt:1', { kind: 'checkin', umore: 6 });
    const row = await v.db.records.get('evt:1');
    expect(Object.keys(row!).sort()).toEqual(['ct', 'id', 'iv', 'v']);
    expect(new TextDecoder().decode(row!.ct)).not.toContain('checkin');
  });

  it('locks, refuses access, unlocks with the right passphrase only', async () => {
    const v = freshVault();
    await v.create('una frase lunga e mia');
    await v.put('evt:1', { umore: 6 });
    v.lock();
    expect(v.unlocked).toBe(false);
    await expect(v.get('evt:1')).rejects.toBeInstanceOf(VaultLockedError);
    await expect(v.unlock('altra frase')).rejects.toBeInstanceOf(WrongPassphraseError);
    await v.unlock('una frase lunga e mia');
    expect(await v.get('evt:1')).toEqual({ umore: 6 });
  });

  it('changes passphrase without losing data', async () => {
    const v = freshVault();
    await v.create('vecchia frase segreta');
    await v.put('evt:1', 'dato');
    await v.changePassphrase('vecchia frase segreta', 'nuova frase segreta');
    v.lock();
    await expect(v.unlock('vecchia frase segreta')).rejects.toBeInstanceOf(WrongPassphraseError);
    await v.unlock('nuova frase segreta');
    expect(await v.get('evt:1')).toBe('dato');
  });

  it('lists by prefix', async () => {
    const v = freshVault();
    await v.create('una frase lunga e mia');
    await v.put('evt:a', 1);
    await v.put('evt:b', 2);
    await v.put('cfg:settings', 3);
    expect((await v.list<number>('evt:')).sort()).toEqual([1, 2]);
  });

  it('destroy removes everything', async () => {
    const v = freshVault();
    await v.create('una frase lunga e mia');
    await v.put('evt:a', 1);
    const name = v.db.name;
    await v.destroy();
    const names = await indexedDB.databases();
    expect(names.map((d) => d.name)).not.toContain(name);
  });
});
