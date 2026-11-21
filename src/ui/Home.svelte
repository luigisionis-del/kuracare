<script lang="ts">
  import { lock, session } from '../lib/session.svelte';

  const today = new Intl.DateTimeFormat('it-IT', { weekday: 'long', day: 'numeric', month: 'long' }).format(new Date());

  function greeting(): string {
    const h = new Date().getHours();
    if (h < 5) return 'Buonanotte';
    if (h < 13) return 'Buongiorno';
    if (h < 18) return 'Buon pomeriggio';
    return 'Buonasera';
  }
</script>

<section class="fade" aria-labelledby="h">
  <p class="muted">{today}</p>
  <h1 id="h">{greeting()}</h1>
  <div class="card">
    <p>Qui troverai le tue registrazioni: come stai, i gesti di cura, le cose da fare.</p>
    <p class="muted">Arrivano nella prossima tappa. Per ora le fondamenta sono pronte: i tuoi dati sono cifrati e restano su questo telefono.</p>
  </div>
  {#if session.settings.checkinWindows.length}
    <div class="card">
      <h2>I tuoi momenti</h2>
      <ul>
        {#each session.settings.checkinWindows as w}<li>dalle {w.start} alle {w.end}</li>{/each}
      </ul>
    </div>
  {/if}
  <div class="actions">
    <button onclick={lock}>Chiudi Kuracare</button>
  </div>
</section>
