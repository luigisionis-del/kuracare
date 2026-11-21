import type { AnyEvent } from './repo';

/**
 * "Se ti va": factual details that can be added later, in a quiet moment.
 * Check-in answers are deliberately excluded: a mood rated hours later is a
 * memory, not a momentary assessment.
 */
export type Detail = 'strain-values' | 'amount' | 'route' | 'fulfilment' | 'duration' | 'meal-what';

export function missingDetails(e: AnyEvent): Detail[] {
  const out: Detail[] = [];
  switch (e.kind) {
    case 'cannabis':
      if (e.payload.thcPct === null || e.payload.cbdPct === null) out.push('strain-values');
      if (e.payload.amount === null) out.push('amount');
      if (e.payload.route === null) out.push('route');
      break;
    case 'supplement':
      if (e.payload.amount === null) out.push('amount');
      break;
    case 'care':
      if (e.payload.fulfilment === null) out.push('fulfilment');
      if (e.payload.durationMin === null) out.push('duration');
      break;
    case 'meal':
      if (!e.payload.description && !e.payload.photoId) out.push('meal-what');
      break;
  }
  return out;
}

export const ENRICH_DAYS = 7;
export const ENRICH_MAX = 5;

export function enrichCandidates(events: AnyEvent[], now = new Date()): AnyEvent[] {
  const since = now.getTime() - ENRICH_DAYS * 86_400_000;
  return events
    .filter((e) => !e.settled && Date.parse(e.ts) >= since && missingDetails(e).length > 0)
    .slice(0, ENRICH_MAX);
}
