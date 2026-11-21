<script lang="ts">
  import { untrack } from 'svelte';
  import { MEAL_TYPES, type MealType } from '../../core/events/kinds';
  import { savePhoto } from '../../core/events/repo';
  import { MEAL_LABELS } from '../../lib/labels';
  import { record, showToast } from '../../lib/diary.svelte';
  import { back } from '../../lib/nav.svelte';
  import { shrinkPhoto } from '../../lib/photo';
  import { vault } from '../../lib/vault';

  let { mealType: initial }: { mealType?: MealType } = $props();

  let mealType = $state<MealType>(untrack(() => initial) ?? 'lunch');
  let description = $state('');
  let photo = $state<string | null>(null);
  let busy = $state(false);

  async function pick(e: Event) {
    const file = (e.currentTarget as HTMLInputElement).files?.[0];
    if (file) photo = await shrinkPhoto(file);
  }

  async function save() {
    busy = true;
    const photoId = photo ? await savePhoto(vault, photo) : null;
    await record('meal', { mealType, description: description.trim() || null, photoId });
    showToast({ text: 'Segnato. Buon appetito, anche a posteriori.' });
    back();
  }
</script>

<section class="fade" aria-labelledby="ml">
  <h1 id="ml">Un pasto</h1>
  <div class="chips" role="group" aria-label="Quale pasto">
    {#each MEAL_TYPES as t}
      <button class="chip" aria-pressed={mealType === t} onclick={() => (mealType = t)}>{MEAL_LABELS[t]}</button>
    {/each}
  </div>
  <label for="md">Cosa c'era, in breve <span class="muted">(facoltativo)</span></label>
  <input id="md" type="text" maxlength="140" autocomplete="off" bind:value={description} />
  <label for="mp">Oppure una foto <span class="muted">(resta solo sul telefono)</span></label>
  <input id="mp" class="file" type="file" accept="image/*" capture="environment" onchange={pick} />
  {#if photo}<img class="photo" src={photo} alt="Foto del pasto" />{/if}
  <div class="actions">
    <button class="primary" disabled={busy} onclick={save}>Segna</button>
    <button class="quiet" onclick={back}>Non ora</button>
  </div>
</section>
