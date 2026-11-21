<script lang="ts">
  import { CHECKIN_ITEMS, emptyCheckin } from '../../core/events/kinds';
  import { CHECKIN_LABELS } from '../../lib/labels';
  import { record, recognition, showToast } from '../../lib/diary.svelte';
  import { back } from '../../lib/nav.svelte';
  import Scale from '../parts/Scale.svelte';

  const answers = $state(emptyCheckin());
  let busy = $state(false);

  async function save() {
    busy = true;
    await record('checkin', $state.snapshot(answers));
    showToast({ text: recognition() });
    back();
  }
</script>

<section class="fade" aria-labelledby="ci">
  <h1 id="ci">Come stai, adesso?</h1>
  <p class="muted">Rispondi solo a ciò che ti va: ogni voce si può lasciare vuota.</p>
  {#each CHECKIN_ITEMS as item}
    <Scale id={`ci-${item}`} {...CHECKIN_LABELS[item]} bind:value={answers[item]} />
  {/each}
  <div class="actions">
    <button class="primary" disabled={busy} onclick={save}>Fatto</button>
    <button class="quiet" onclick={back}>Non ora</button>
  </div>
</section>
