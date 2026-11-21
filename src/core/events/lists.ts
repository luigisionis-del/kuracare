import type { Route } from './kinds';
import type { Vault } from '../store/vault';

/** User-editable presets and labels. Values come from Luigi, never invented. */
export interface StrainPreset {
  id: string;
  name: string;
  group: 'thc' | 'cbd';
  thcPct: number | null;
  cbdPct: number | null;
  amount: number | null;
  unit: string;
  route: Route | null;
}

export interface SupplementPreset {
  id: string;
  name: string;
  amount: number | null;
  unit: string;
}

export interface Lists {
  version: 1;
  strains: StrainPreset[];
  supplements: SupplementPreset[];
  careLabels: string[];
  thoughtLabels: string[];
}

export const LISTS_ID = 'cfg:lists';

const strain = (id: string, name: string, group: 'thc' | 'cbd'): StrainPreset => ({
  id, name, group, thcPct: null, cbdPct: null, amount: null, unit: 'g', route: null,
});
const supp = (id: string, name: string): SupplementPreset => ({ id, name, amount: null, unit: 'mg' });

export function defaultLists(): Lists {
  return {
    version: 1,
    strains: [
      strain('s-sherbinskis', 'Sherbinskis × Silent Seeds', 'thc'),
      strain('s-mandarine', 'Mandarine Flokies', 'cbd'),
      strain('s-lemon', 'Lemon Cockies', 'cbd'),
    ],
    supplements: [supp('i-cbd', 'CBD'), supp('i-agpc', 'Alfa-GPC'), supp('i-glicina', 'Glicina'), supp('i-colina', 'Colina')],
    careLabels: [
      'Passeggiata', 'Movimento', 'Pratica corporea', 'Natura', 'Riposo', 'Sonno buono', 'Un buon pasto', 'Contatto con le persone',
    ],
    thoughtLabels: ['Rimuginio', 'Preoccupazione', 'Autocritica', 'Un ricordo che torna', 'Altro'],
  };
}

export async function loadLists(vault: Vault): Promise<Lists> {
  return (await vault.get<Lists>(LISTS_ID)) ?? defaultLists();
}

export async function saveLists(vault: Vault, lists: Lists): Promise<void> {
  await vault.put(LISTS_ID, lists);
}

export function newId(prefix: string): string {
  return `${prefix}-${crypto.randomUUID().slice(0, 8)}`;
}
