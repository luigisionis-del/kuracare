<script lang="ts">
  import { diary, record, showToast } from '../../lib/diary.svelte';
  import { back } from '../../lib/nav.svelte';

  let label = $state<string | null>(null);
  let note = $state('');
  let busy = $state(false);

  async function save() {
    busy = true;
    await record('thought', { present: true, label, note: note.trim() || null });
    showToast({ text: 'Annotato. Lasciarlo qui può già alleggerire.' });
    back();
  }
</script>

<section class="fade" aria-labelledby="th">
  <h1 id="th">Un pensiero che torna</h1>
  <p class="muted">Se vuoi, dagli un nome. Non serve altro.</p>
  <div class="chips" role="group" aria-label="Che tipo di pensiero">
    {#each diary.lists.thoughtLabels as l}
      <button class="chip" aria-pressed={label === l} onclick={() => (label = label === l ? null : l)}>{l}</button>
    {/each}
  </div>
  <label for="tn">Due parole, se ti va <span class="muted">(restano solo qui)</span></label>
  <input id="tn" type="text" maxlength="280" autocomplete="off" bind:value={note} />
  <div class="actions">
    <button class="primary" disabled={busy} onclick={save}>Segna</button>
    <button class="quiet" onclick={back}>Non ora</button>
  </div>
</section>
