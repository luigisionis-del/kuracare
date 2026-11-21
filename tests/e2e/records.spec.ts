import { test, expect, onboard } from './fixtures';

test('check-in: partial answers, back home with a recognition phrase', async ({ page, external, cspViolations }) => {
  await onboard(page);
  const start = Date.now();
  await page.getByRole('button', { name: 'Fai il check-in' }).click();
  await page.getByLabel('Umore').fill('7');
  await page.getByLabel('Ansia').fill('3');
  await page.getByRole('button', { name: 'Fatto' }).click();
  expect(Date.now() - start).toBeLessThan(10_000);
  await expect(page.getByRole('status')).toBeVisible();
  await expect(page.getByRole('button', { name: /Check-in \(2 risposte su 5\)/ })).toBeVisible();
});

test('check-in can be skipped entirely with "Non ora"', async ({ page, external, cspViolations }) => {
  await onboard(page);
  await page.getByRole('button', { name: 'Fai il check-in' }).click();
  await page.getByRole('button', { name: 'Non ora' }).click();
  await expect(page.getByRole('heading', { name: /Buon|Buonanotte/ })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Oggi' })).toHaveCount(0);
});

test('care gesture: one tap, then optional fulfilment', async ({ page, external, cspViolations }) => {
  await onboard(page);
  await page.getByRole('button', { name: 'Passeggiata' }).click();
  await page.getByRole('button', { name: 'Quanto ti ha fatto bene?' }).click();
  await page.getByLabel('Quanto ti ha fatto bene?').fill('8');
  await page.getByRole('button', { name: '30', exact: true }).click();
  await page.getByRole('button', { name: 'Salva' }).click();
  await expect(page.getByRole('button', { name: /Passeggiata · 30 min · appagamento 8/ })).toBeVisible();
});

test('cannabis: one tap from preset, time and details editable afterwards', async ({ page, external, cspViolations }) => {
  await onboard(page);
  await page.getByRole('button', { name: 'Mandarine Flokies' }).click();
  await expect(page.getByRole('status')).toContainText('Mandarine Flokies: segnato alle');
  await page.getByRole('status').getByRole('button', { name: 'Modifica' }).click();
  const when = page.getByLabel('Quando');
  const value = await when.inputValue();
  await when.fill(value.slice(0, 11) + '07:15');
  await page.getByLabel('CBD %').fill('12,5');
  await page.getByLabel('Quantità').fill('0,1');
  await page.getByRole('button', { name: 'Vaporizzazione' }).click();
  await page.getByRole('button', { name: 'Salva' }).click();
  const entry = page.getByRole('button', { name: /07:15 Mandarine Flokies · 0,1 g · vaporizzazione/ });
  await expect(entry).toBeVisible();
});

test('supplement, meal and thought', async ({ page, external, cspViolations }) => {
  await onboard(page);
  await page.getByRole('button', { name: 'Glicina' }).click();
  await page.getByRole('button', { name: 'Colazione' }).first().click();
  await page.getByLabel(/Cosa c'era/).fill('pane e miele');
  await page.getByRole('button', { name: 'Segna' }).click();
  await page.getByRole('button', { name: 'Un pensiero che torna' }).click();
  await page.getByRole('button', { name: 'Rimuginio' }).click();
  await page.getByRole('button', { name: 'Segna' }).click();
  const today = page.locator('ul.today');
  await expect(today).toContainText('Glicina');
  await expect(today).toContainText('Colazione · pane e miele');
  await expect(today).toContainText('Pensiero: rimuginio');
});

test('"Se ti va": add details or let it be', async ({ page, external, cspViolations }) => {
  await onboard(page);
  await page.getByRole('button', { name: 'Natura' }).click();
  await page.getByRole('button', { name: 'Riposo' }).click();
  await page.getByRole('button', { name: /Se ti va/ }).click();
  await expect(page.locator('.card').filter({ hasText: 'Natura' })).toBeVisible();
  await page.locator('.card').filter({ hasText: 'Natura' }).getByRole('button', { name: 'Va bene così' }).click();
  await expect(page.locator('.card').filter({ hasText: 'Natura' })).toHaveCount(0);
  await expect(page.locator('.card').filter({ hasText: 'Riposo' })).toBeVisible();
});

test('lists: values copied from the label are used by the one-tap button', async ({ page, external, cspViolations }) => {
  await onboard(page);
  await page.getByRole('button', { name: /Se ti va/ }).click();
  await page.getByRole('button', { name: 'Apri le tue liste' }).click();
  const card = page.getByRole('group', { name: 'Lemon Cockies' });
  await card.getByLabel('THC %').fill('0,2');
  await card.getByLabel('Quantità abituale').fill('0,15');
  await card.getByRole('button', { name: 'Orale' }).click();
  await page.getByRole('button', { name: 'Salva le liste' }).click();
  await page.getByRole('button', { name: 'Lemon Cockies' }).click();
  await expect(page.locator('ul.today')).toContainText('Lemon Cockies · 0,15 g · orale');
});

test('evening: reflection stays on the device, meal question has "Non ora"', async ({ page, external, cspViolations }) => {
  await page.clock.setFixedTime(new Date('2026-11-21T20:30:00'));
  await onboard(page);
  await page.getByLabel('Cosa ti ha fatto bene oggi').fill('il sole sul terrazzo');
  await page.getByRole('button', { name: 'Tienilo' }).click();
  await expect(page.getByText('il sole sul terrazzo')).toBeVisible();
  await expect(page.getByText('Ti va di segnare cosa hai mangiato oggi?')).toBeVisible();
  await page.locator('.card.soft').filter({ hasText: 'mangiato' }).getByRole('button', { name: 'Non ora' }).click();
  await expect(page.getByText('Ti va di segnare cosa hai mangiato oggi?')).toHaveCount(0);
});

test('android back gesture returns home', async ({ page, external, cspViolations }) => {
  await onboard(page);
  await page.getByRole('button', { name: 'Fai il check-in' }).click();
  await page.goBack();
  await expect(page.getByRole('button', { name: 'Fai il check-in' })).toBeVisible();
});

test('records survive lock and unlock', async ({ page, external, cspViolations }) => {
  await onboard(page);
  await page.getByRole('button', { name: 'Movimento' }).click();
  await page.getByRole('button', { name: 'Chiudi Kuracare' }).click();
  await page.getByLabel('Passphrase').fill('mare vento pane sole');
  await page.getByRole('button', { name: 'Apri' }).click();
  await expect(page.locator('ul.today')).toContainText('Movimento', { timeout: 20_000 });
});
