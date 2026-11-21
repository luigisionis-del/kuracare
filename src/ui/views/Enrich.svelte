<script lang="ts">
  import { enrichCandidates } from '../../core/events/enrich';
  import { hhmm } from '../../core/model/localtime';
  import { describe } from '../../lib/labels';
  import { change, diary } from '../../lib/diary.svelte';
  import { back, go } from '../../lib/nav.svelte';

  const candidates = $derived(enrichCandidates(diary.events));
  const dayFmt = new Intl.DateTimeFormat('it-IT', { weekday: 'long', day: 'numeric' });
</script>

<section class="fade" aria-labelledby="en">
  <h1 id="en">Se ti va</h1>
  <p class="muted">Un angolo per i momenti tranquilli. Qui puoi aggiungere qualche dettaglio a ciò che hai segnato di fretta. Nulla è obbligatorio.</p>

  {#each candidates as e (e.id)}
    <div class="card">
      <p><strong>{describe(e)}</strong><br /><span class="muted">{dayFmt.format(new Date(e.ts))}, {hhmm(e.ts)}</span></p>
      <div class="row">
        <button class="small" onclick={() => go({ name: 'edit', id: e.id })}>Aggiungi dettagli</button>
        <button class="quiet small" onclick={() => change({ ...e, settled: true })}>Va bene così</button>
      </div>
    </div>
  {:else}
    <div class="card"><p>Qui non c'è niente da sistemare. Goditi il momento.</p></div>
  {/each}

  <h2>Le tue liste</h2>
  <p class="muted">Varietà, integratori, gesti di cura, pensieri: puoi riordinarli con calma, per esempio copiando THC e CBD dalle etichette.</p>
  <button onclick={() => go({ name: 'lists' })}>Apri le tue liste</button>

  <div class="actions"><button class="quiet" onclick={back}>Torna a oggi</button></div>
</section>
