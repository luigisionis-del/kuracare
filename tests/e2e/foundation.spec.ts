import { test, expect, onboard, PASSPHRASE } from './fixtures';

test('first run: nothing can be saved without a passphrase', async ({ page, external, cspViolations }) => {
  await page.goto('/');
  await expect(page.getByRole('heading', { name: /benvenuto/i })).toBeVisible();
  await page.getByRole('button', { name: 'Avanti' }).click();
  await page.getByRole('button', { name: 'Avanti' }).click();
  await expect(page.getByText(/non si possono recuperare/).first()).toBeVisible();
  await page.getByRole('button', { name: 'Scelgo la passphrase' }).click();
  const next = page.getByRole('button', { name: 'Avanti' });
  await page.getByLabel('Passphrase', { exact: true }).fill('corta');
  await expect(next).toBeDisabled();
  await page.getByLabel('Passphrase', { exact: true }).fill(PASSPHRASE);
  await page.getByLabel('Scrivila di nuovo').fill(PASSPHRASE);
  await expect(next).toBeDisabled(); // must acknowledge it cannot be recovered
  await page.getByLabel(/Ho capito/).check();
  await expect(next).toBeEnabled();
  // The (empty) database may exist; it must hold neither a key nor records.
  const counts = await page.evaluate(
    () =>
      new Promise<number[]>((resolve, reject) => {
        const req = indexedDB.open('kuracare');
        req.onerror = () => reject(req.error);
        req.onsuccess = () => {
          const db = req.result;
          if (!db.objectStoreNames.contains('records')) return resolve([0, 0]);
          const tx = db.transaction(['vault', 'records'], 'readonly');
          const a = tx.objectStore('vault').count();
          const b = tx.objectStore('records').count();
          tx.oncomplete = () => resolve([a.result, b.result]);
        };
      }),
  );
  expect(counts).toEqual([0, 0]);
});

test('onboarding, lock and unlock', async ({ page, external, cspViolations }) => {
  await onboard(page, { window: true });

  await page.getByRole('button', { name: 'Chiudi Kuracare' }).click();
  await expect(page.getByRole('heading', { name: 'Che bello rivederti' })).toBeVisible();

  await page.getByLabel('Passphrase').fill('non è questa');
  await page.getByRole('button', { name: 'Apri' }).click();
  await expect(page.getByRole('alert')).toContainText('Non sembra la passphrase giusta');

  await page.getByLabel('Passphrase').fill(PASSPHRASE);
  await page.getByRole('button', { name: 'Apri' }).click();
  await expect(page.getByRole('button', { name: 'Fai il check-in' })).toBeVisible({ timeout: 20_000 });
});

test('reload always starts locked', async ({ page, external, cspViolations }) => {
  await onboard(page);
  await page.reload();
  await expect(page.getByRole('heading', { name: 'Che bello rivederti' })).toBeVisible();
});

test('stored data is ciphertext only', async ({ page, external, cspViolations }) => {
  await onboard(page, { window: true });
  const dump = await page.evaluate(
    () =>
      new Promise<string>((resolve, reject) => {
        const req = indexedDB.open('kuracare');
        req.onerror = () => reject(req.error);
        req.onsuccess = () => {
          const tx = req.result.transaction('records', 'readonly');
          const all = tx.objectStore('records').getAll();
          all.onsuccess = () =>
            resolve(all.result.map((r: { id: string; ct: Uint8Array }) => r.id + new TextDecoder().decode(r.ct)).join('|'));
        };
      }),
  );
  expect(dump).toContain('cfg:settings');
  expect(dump).not.toContain('09:00');
  expect(dump).not.toContain('checkinWindows');
});

test('forgotten passphrase: wipe needs explicit confirmation', async ({ page, external, cspViolations }) => {
  await onboard(page);
  await page.getByRole('button', { name: 'Chiudi Kuracare' }).click();
  await page.getByRole('button', { name: 'Ho dimenticato la passphrase' }).click();
  const wipe = page.getByRole('button', { name: 'Cancella tutto e ricomincia' });
  await expect(wipe).toBeDisabled();
  await page.getByLabel(/Per cancellare tutto/).fill('CANCELLA');
  await wipe.click();
  await expect(page.getByRole('heading', { name: /benvenuto/i })).toBeVisible();
});

test('installable: manifest and service worker', async ({ page, external, cspViolations }) => {
  await page.goto('/');
  const manifest = await (await page.request.get('/manifest.webmanifest')).json();
  expect(manifest.lang).toBe('it');
  expect(manifest.display).toBe('standalone');
  expect(manifest.icons.some((i: { purpose?: string }) => i.purpose === 'maskable')).toBe(true);
  await expect
    .poll(() => page.evaluate(async () => !!(await navigator.serviceWorker.getRegistration())), { timeout: 15_000 })
    .toBe(true);
});
