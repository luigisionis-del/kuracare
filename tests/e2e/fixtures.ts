import { test as base, expect, type Page } from '@playwright/test';

/**
 * Every test fails if the page tries to reach anything outside its own origin
 * or triggers a Content Security Policy violation.
 */
export const test = base.extend<{ external: string[]; cspViolations: string[] }>({
  external: async ({ page, baseURL }, use) => {
    const origin = new URL(baseURL!).origin;
    const hits: string[] = [];
    await page.route('**/*', (route) => {
      const url = route.request().url();
      if (url.startsWith(origin) || url.startsWith('data:') || url.startsWith('blob:')) return route.continue();
      hits.push(url);
      return route.abort();
    });
    await use(hits);
    expect(hits, 'requests to external domains').toEqual([]);
  },
  cspViolations: async ({ page }, use) => {
    const v: string[] = [];
    page.on('console', (m) => {
      if (/Content Security Policy|Refused to/i.test(m.text())) v.push(m.text());
    });
    await use(v);
    expect(v, 'CSP violations').toEqual([]);
  },
});

export { expect };

export const PASSPHRASE = 'mare vento pane sole';

export async function onboard(page: Page, opts: { window?: boolean } = {}) {
  await page.goto('/');
  await page.getByRole('button', { name: 'Avanti' }).click();
  await page.getByRole('button', { name: 'Avanti' }).click();
  await page.getByRole('button', { name: 'Scelgo la passphrase' }).click();
  await page.getByLabel('Passphrase', { exact: true }).fill(PASSPHRASE);
  await page.getByLabel('Scrivila di nuovo').fill(PASSPHRASE);
  await page.getByLabel(/Ho capito/).check();
  await page.getByRole('button', { name: 'Avanti' }).click();
  if (opts.window) {
    await page.getByRole('button', { name: 'Aggiungi un momento' }).click();
    await page.getByRole('button', { name: 'Iniziamo', exact: true }).click();
  } else {
    await page.getByRole('button', { name: /Per ora nessuno/ }).click();
  }
  await expect(page.getByRole('button', { name: 'Chiudi Kuracare' })).toBeVisible({ timeout: 20_000 });
}
