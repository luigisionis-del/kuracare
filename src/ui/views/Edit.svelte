<script lang="ts">
  import { untrack } from 'svelte';
  import { CHECKIN_ITEMS, ROUTES, MEAL_TYPES, positive } from '../../core/events/kinds';
  import { loadPhoto, savePhoto, type AnyEvent } from '../../core/events/repo';
  import { fromLocalInput, toLocalInput } from '../../core/model/localtime';
  import { CHECKIN_LABELS, MEAL_LABELS, ROUTE_LABELS, describe } from '../../lib/labels';
  import { change, diary, findEvent, forget } from '../../lib/diary.svelte';
  import { back } from '../../lib/nav.svelte';
  import { shrinkPhoto } from '../../lib/photo';
  import { vault } from '../../lib/vault';
  import Scale from '../parts/Scale.svelte';

  let { id }: { id: string } = $props();

  // Keyed by id in App.svelte: the view is recreated when the id changes.
  const original = untrack(() => findEvent(id));
  let draft = $state<AnyEvent | null>(original ? structuredClone($state.snapshot(original)) as AnyEvent : null);
  let when = $state(original ? toLocalInput(original.ts) : '');
  let photo = $state<string | null>(null);
  let confirmDelete = $state(false);
  let busy = $state(false);

  // Number fields are edited as text so that "0,5" works with an Italian keyboard.
  let amountText = $state('');
  let thcText = $state('');
  let cbdText = $state('');
  let durationText = $state('');
  const show = (n: number | null) => (n === null ? '' : String(n).replace('.', ','));

  if (draft?.kind === 'cannabis') {
    amountText = show(draft.payload.amount);
    thcText = show(draft.payload.thcPct);
    cbdText = show(draft.payload.cbdPct);
  } else if (draft?.kind === 'supplement') amountText = show(draft.payload.amount);
  else if (draft?.kind === 'care') durationText = show(draft.payload.durationMin);
  if (draft?.kind === 'meal' && draft.payload.photoId) loadPhoto(vault, draft.payload.photoId).then((p) => (photo = p ?? null));

  async function pick(e: Event) {
    const file = (e.currentTarget as HTMLInputElement).files?.[0];
    if (file && draft?.kind === 'meal') {
      photo = await shrinkPhoto(file);
      draft.payload.photoId = await savePhoto(vault, photo);
    }
  }

  async function save() {
    if (!draft) return;
    busy = true;
    const ts = fromLocalInput(when);
    if (ts) draft.ts = ts;
    if (draft.kind === 'cannabis') {
      draft.payload.amount = positive(amountText);
      draft.payload.thcPct = positive(thcText, 100);
      draft.payload.cbdPct = positive(cbdText, 100);
      draft.payload.strain = draft.payload.strain.trim() || 'Senza nome';
    } else if (draft.kind === 'supplement') {
      draft.payload.amount = positive(amountText);
    } else if (draft.kind === 'care') {
      draft.payload.durationMin = positive(durationText, 24 * 60);
    } else if (draft.kind === 'meal') {
      draft.payload.description = draft.payload.description?.trim() || null;
    } else if (draft.kind === 'thought') {
      draft.payload.note = draft.payload.note?.trim() || null;
    }
    await change(draft);
    back();
  }

  async function remove() {
    if (!draft) return;
    await forget(draft);
    back();
  }
</script>

{#if !draft}
  <p class="muted">Questa registrazione non c'è più.</p>
  <button onclick={back}>Torna indietro</button>
{:else}
  <section class="fade" aria-labelledby="ed">
    <h1 id="ed">{describe(draft)}</h1>

    <label for="when">Quando</label>
    <input id="when" type="datetime-local" bind:value={when} />

    {#if draft.kind === 'checkin'}
      <p class="hint">Le risposte del check-in restano com'erano in quel momento: puoi cambiarne solo l'ora.</p>
      <ul class="plain">
        {#each CHECKIN_ITEMS as k}<li>{CHECKIN_LABELS[k].label}: {draft.payload[k] ?? '–'}</li>{/each}
      </ul>
    {:else if draft.kind === 'care'}
      <Scale id="ful" label="Quanto ti ha fatto bene?" low="poco" high="moltissimo" bind:value={draft.payload.fulfilment} />
      <label for="dur">Quanto è durato <span class="muted">(minuti)</span></label>
      <div class="chips">
        {#each [5, 10, 20, 30, 60] as m}
          <button class="chip" aria-pressed={durationText === String(m)} onclick={() => (durationText = durationText === String(m) ? '' : String(m))}>{m}</button>
        {/each}
      </div>
      <input id="dur" type="text" inputmode="numeric" bind:value={durationText} />
    {:else if draft.kind === 'cannabis'}
      <label for="st">Varietà</label>
      <input id="st" type="text" bind:value={draft.payload.strain} />
      <div class="grid2">
        <div><label for="thc">THC %</label><input id="thc" type="text" inputmode="decimal" bind:value={thcText} /></div>
        <div><label for="cbd">CBD %</label><input id="cbd" type="text" inputmode="decimal" bind:value={cbdText} /></div>
        <div><label for="am">Quantità</label><input id="am" type="text" inputmode="decimal" bind:value={amountText} /></div>
        <div><label for="un">Unità</label><input id="un" type="text" bind:value={draft.payload.unit} /></div>
      </div>
      <p class="label">Via di assunzione</p>
      <div class="chips" role="group" aria-label="Via di assunzione">
        {#each ROUTES as r}
          {@const d = draft}
          <button class="chip" aria-pressed={d.payload.route === r} onclick={() => (d.payload.route = d.payload.route === r ? null : r)}>{ROUTE_LABELS[r]}</button>
        {/each}
      </div>
    {:else if draft.kind === 'supplement'}
      <div class="grid2">
        <div><label for="am">Quantità</label><input id="am" type="text" inputmode="decimal" bind:value={amountText} /></div>
        <div><label for="un">Unità</label><input id="un" type="text" bind:value={draft.payload.unit} /></div>
      </div>
    {:else if draft.kind === 'meal'}
      <div class="chips" role="group" aria-label="Quale pasto">
        {#each MEAL_TYPES as t}
          {@const d = draft}
          <button class="chip" aria-pressed={d.payload.mealType === t} onclick={() => (d.payload.mealType = t)}>{MEAL_LABELS[t]}</button>
        {/each}
      </div>
      <label for="md">Cosa c'era</label>
      <input id="md" type="text" maxlength="140" value={draft.payload.description ?? ''} oninput={(e) => { if (draft?.kind === 'meal') draft.payload.description = e.currentTarget.value; }} />
      <label for="mp">Foto <span class="muted">(resta solo sul telefono)</span></label>
      <input id="mp" class="file" type="file" accept="image/*" capture="environment" onchange={pick} />
      {#if photo}<img class="photo" src={photo} alt="Foto del pasto" />{/if}
    {:else if draft.kind === 'thought'}
      <div class="chips" role="group" aria-label="Che tipo di pensiero">
        {#each diary.lists.thoughtLabels as l}
          {@const d = draft}
          <button class="chip" aria-pressed={d.payload.label === l} onclick={() => (d.payload.label = d.payload.label === l ? null : l)}>{l}</button>
        {/each}
      </div>
      <label for="tn">Due parole <span class="muted">(restano solo qui)</span></label>
      <input id="tn" type="text" maxlength="280" value={draft.payload.note ?? ''} oninput={(e) => { if (draft?.kind === 'thought') draft.payload.note = e.currentTarget.value; }} />
    {:else if draft.kind === 'reflection'}
      <label for="rf">Cosa ti ha fatto bene</label>
      <input id="rf" type="text" maxlength="200" bind:value={draft.payload.text} />
    {/if}

    <div class="actions">
      <button class="primary" disabled={busy} onclick={save}>Salva</button>
      <button class="quiet" onclick={back}>Lascia com'era</button>
      {#if !confirmDelete}
        <button class="quiet" onclick={() => (confirmDelete = true)}>Elimina questa registrazione</button>
      {:else}
        <button class="danger" onclick={remove}>Sì, eliminala</button>
      {/if}
    </div>
  </section>
{/if}
