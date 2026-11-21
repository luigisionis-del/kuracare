import type { MealType } from '../core/events/kinds';
import { diary } from './diary.svelte';

export type View =
  | { name: 'home' }
  | { name: 'checkin' }
  | { name: 'thought' }
  | { name: 'meal'; mealType?: MealType }
  | { name: 'edit'; id: string }
  | { name: 'enrich' }
  | { name: 'lists' };

export const nav = $state<{ view: View }>({ view: { name: 'home' } });

/** Sub-views push a history entry so Android's back gesture returns home. */
export function go(view: View): void {
  if (nav.view.name === 'home') history.pushState({ kuracare: true }, '');
  else history.replaceState({ kuracare: true }, '');
  nav.view = view;
  diary.toast = null;
  window.scrollTo(0, 0);
}

export function back(): void {
  if (history.state?.kuracare) history.back();
  else nav.view = { name: 'home' };
}

export function resetNav(): void {
  nav.view = { name: 'home' };
}

if (typeof window !== 'undefined') {
  window.addEventListener('popstate', () => {
    nav.view = { name: 'home' };
  });
}
