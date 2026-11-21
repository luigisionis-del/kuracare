/** Parses the editable phrases file: one phrase per line, `#` starts a comment. */
export function parsePhrases(text: string): string[] {
  return text
    .split(/\r?\n/)
    .map((l) => l.trim())
    .filter((l) => l && !l.startsWith('#'));
}

/** Random phrase, never the same as the previous one when there is a choice. */
export function pickPhrase(phrases: string[], previous: string | null, rnd = Math.random): string | null {
  if (!phrases.length) return null;
  const pool = phrases.length > 1 ? phrases.filter((p) => p !== previous) : phrases;
  return pool[Math.floor(rnd() * pool.length)] ?? null;
}
