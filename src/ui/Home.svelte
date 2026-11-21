<script lang="ts">
  import { MEAL_TYPES, type MealType } from '../core/events/kinds';
  import type { StrainPreset, SupplementPreset } from '../core/events/lists';
  import { hhmm, isSameLocalDay } from '../core/model/localtime';
  import { diary, record, recognition, showToast } from '../lib/diary.svelte';
  import { describe, MEAL_LABELS } from '../lib/labels';
  import { go } from '../lib/nav.svelte';
  import { lock, session, updateSettings } from '../lib/session.svelte';

  const now = new Date();
  const todayKey = `${now.getFullYear()}-${now.getMonth() + 1}-${now.getDate()}`;
  const today = new Intl.DateTimeFormat('it-IT', { weekday: 'long', day: 'numeric', month: 'long' }).format(now);
  const evening = now.getHours() >= 18;
  const paused = $derived(!!session.settings.pauseUntil && Date.parse(session.settings.pauseUntil) > Date.now());

  const todays = $derived(diary.events.filter((e) => isSameLocalDay(e.ts, now)));
  const reflection = $derived(todays.find((e) => e.kind === 'reflection'));
  const mealsLeft = $derived(
    (['breakfast', 'lunch', 'dinner'] as MealType[]).filter((t) => !todays.some((e) => e.kind === 'meal' && e.payload.mealType === t)),
  );
  const showMealRecall = $derived(evening && !paused && mealsLeft.length > 0 && session.settings.mealRecallDismissedOn !== todayKey);
  const thcStrains = $derived(diary.lists.strains.filter((s) => s.group === 'thc'));
  const cbdStrains = $derived(diary.lists.strains.filter((s) => s.group === 'cbd'));

  let reflectionText = $state('');

  function greeting(): string {
    const h = now.getHours();
    if (h < 5) return 'Buonanotte';
    if (h < 13) return 'Buongiorno';
    if (h < 18) return 'Buon pomeriggio';
    return 'Buonasera';
  }

  async function care(label: string) {
    const e = await record('care', { label, durationMin: null, fulfilment: null });
    showToast({ text: recognition(), action: { label: 'Quanto ti ha fatto bene?', eventId: e.id } }, 8000);
  }

  async function strain(s: StrainPreset) {
    const e = await record('cannabis', {
      presetId: s.id, strain: s.name, thcPct: s.thcPct, cbdPct: s.cbdPct, amount: s.amount, unit: s.unit, route: s.route,
    });
    showToast({ text: `${s.name}: segnato alle ${hhmm(e.ts)}.`, action: { label: 'Modifica', eventId: e.id } });
  }

  async function supplement(s: SupplementPreset) {
    const e = await record('supplement', { presetId: s.id, name: s.name, amount: s.amount, unit: s.unit });
    showToast({ text: `${s.name}: segnato alle ${hhmm(e.ts)}.`, action: { label: 'Modifica', eventId: e.id } });
  }

  async function saveReflection(e: SubmitEvent) {
    e.preventDefault();
    const text = reflectionText.trim();
    if (!text) return;
    await record('reflection', { text });
    reflectionText = '';
    showToast({ text: recognition() });
  }
</script>

<section class="fade" aria-labelledby="h">
  <p class="muted">{today}</p>
  <h1 id="h">{greeting()}</h1>

  <div class="card">
    <h2 class="first">Come stai?</h2>
    <p class="muted">Cinque domande veloci, tutte facoltative.</p>
    <button class="primary wide" onclick={() => go({ name: 'checkin' })}>Fai il check-in</button>
  </div>

  <h2>Un gesto di cura</h2>
  <p class="muted">Un tocco per segnare qualcosa di buono che hai fatto per te.</p>
  <div class="chips">
    {#each diary.lists.careLabels as l}<button class="chip" onclick={() => care(l)}>{l}</button>{/each}
  </div>

  <h2>Assunzioni</h2>
  {#if thcStrains.length}
    <p class="label">Prevalenza THC</p>
    <div class="chips">{#each thcStrains as s}<button class="chip" onclick={() => strain(s)}>{s.name}</button>{/each}</div>
  {/if}
  {#if cbdStrains.length}
    <p class="label">Prevalenza CBD</p>
    <div class="chips">{#each cbdStrains as s}<button class="chip" onclick={() => strain(s)}>{s.name}</button>{/each}</div>
  {/if}
  {#if diary.lists.supplements.length}
    <p class="label">Integratori</p>
    <div class="chips">{#each diary.lists.supplements as s}<button class="chip" onclick={() => supplement(s)}>{s.name}</button>{/each}</div>
  {/if}

  <h2>Pasti</h2>
  <div class="chips">
    {#each MEAL_TYPES as t}<button class="chip" onclick={() => go({ name: 'meal', mealType: t })}>{MEAL_LABELS[t]}</button>{/each}
  </div>

  <div class="row spaced">
    <button onclick={() => go({ name: 'thought' })}>Un pensiero che torna</button>
  </div>

  {#if showMealRecall}
    <div class="card soft">
      <p>Ti va di segnare cosa hai mangiato oggi?</p>
      <div class="chips">
        {#each mealsLeft as t}<button class="chip" onclick={() => go({ name: 'meal', mealType: t })}>{MEAL_LABELS[t]}</button>{/each}
      </div>
      <button class="quiet small" onclick={() => updateSettings({ mealRecallDismissedOn: todayKey })}>Non ora</button>
    </div>
  {/if}

  {#if evening}
    <div class="card soft">
      <h2 class="first">Cosa ti ha fatto bene oggi?</h2>
      {#if reflection && reflection.kind === 'reflection'}
        <p>{reflection.payload.text}</p>
        <button class="quiet small" onclick={() => go({ name: 'edit', id: reflection.id })}>Cambia</button>
      {:else}
        <form onsubmit={saveReflection}>
          <input type="text" maxlength="200" aria-label="Cosa ti ha fatto bene oggi" placeholder="Una riga, se ti va" bind:value={reflectionText} />
          <p class="hint">Resta solo su questo telefono.</p>
          <button class="small" type="submit" disabled={!reflectionText.trim()}>Tienilo</button>
        </form>
      {/if}
    </div>
  {/if}

  {#if todays.length}
    <h2>Oggi</h2>
    <ul class="today">
      {#each todays as e (e.id)}
        <li><button class="entry" onclick={() => go({ name: 'edit', id: e.id })}><span class="time">{hhmm(e.ts)}</span> {describe(e)}</button></li>
      {/each}
    </ul>
  {/if}

  <div class="actions">
    <button onclick={() => go({ name: 'enrich' })}>Se ti va: un momento per sistemare</button>
    <button class="quiet" onclick={lock}>Chiudi Kuracare</button>
  </div>
</section>
