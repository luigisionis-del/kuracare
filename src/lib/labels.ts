import type { CheckinItem, MealType, Route } from '../core/events/kinds';
import type { AnyEvent } from '../core/events/repo';

/**
 * core-v0.1 item labels and anchors. PROVISIONAL: will be replaced by items
 * chosen from the ESM Item Repository (https://osf.io/kg376).
 */
export const CHECKIN_LABELS: Record<CheckinItem, { label: string; low: string; high: string }> = {
  mood: { label: 'Umore', low: 'giù', high: 'bene' },
  energy: { label: 'Energia', low: 'scarica', high: 'piena' },
  pain: { label: 'Dolore', low: 'nessuno', high: 'molto' },
  mentalLoad: { label: 'Pesantezza mentale', low: 'leggera', high: 'pesante' },
  anxiety: { label: 'Ansia', low: 'calma', high: 'molta' },
};

export const ROUTE_LABELS: Record<Route, string> = {
  inhalation: 'Inalazione',
  vaporization: 'Vaporizzazione',
  oral: 'Orale',
  sublingual: 'Sublinguale',
};

export const MEAL_LABELS: Record<MealType, string> = {
  breakfast: 'Colazione',
  lunch: 'Pranzo',
  dinner: 'Cena',
  snack: 'Spuntino',
};

function num(n: number | null, unit = ''): string {
  return n === null ? '' : `${String(n).replace('.', ',')}${unit}`;
}

/** One short line describing an event, for lists. */
export function describe(e: AnyEvent): string {
  switch (e.kind) {
    case 'checkin': {
      const n = (Object.keys(CHECKIN_LABELS) as CheckinItem[]).filter((k) => e.payload[k] !== null).length;
      return n ? `Check-in (${n} risposte su 5)` : 'Check-in';
    }
    case 'care':
      return [e.payload.label, e.payload.durationMin !== null ? `${e.payload.durationMin} min` : '', e.payload.fulfilment !== null ? `appagamento ${e.payload.fulfilment}` : '']
        .filter(Boolean).join(' · ');
    case 'thought':
      return e.payload.label ? `Pensiero: ${e.payload.label.toLowerCase()}` : 'Un pensiero che torna';
    case 'reflection':
      return 'Cosa ti ha fatto bene';
    case 'cannabis':
      return [e.payload.strain, num(e.payload.amount, ` ${e.payload.unit}`), e.payload.route ? ROUTE_LABELS[e.payload.route].toLowerCase() : '']
        .filter(Boolean).join(' · ');
    case 'supplement':
      return [e.payload.name, num(e.payload.amount, ` ${e.payload.unit}`)].filter(Boolean).join(' · ');
    case 'meal':
      return [MEAL_LABELS[e.payload.mealType], e.payload.description ?? (e.payload.photoId ? 'foto' : '')].filter(Boolean).join(' · ');
  }
}
