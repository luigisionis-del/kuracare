/** ISO 8601 local time with explicit UTC offset, e.g. 2026-11-21T09:30:00.000+01:00 */
export function isoWithOffset(date: Date = new Date()): string {
  const pad = (n: number, w = 2) => String(Math.trunc(Math.abs(n))).padStart(w, '0');
  const offsetMin = -date.getTimezoneOffset();
  const sign = offsetMin >= 0 ? '+' : '-';
  return (
    `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}` +
    `T${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(date.getSeconds())}.${pad(date.getMilliseconds(), 3)}` +
    `${sign}${pad(offsetMin / 60)}:${pad(offsetMin % 60)}`
  );
}

/** IANA time-zone name of the device, e.g. "Europe/Rome". */
export function deviceTimeZone(): string {
  return Intl.DateTimeFormat().resolvedOptions().timeZone;
}
