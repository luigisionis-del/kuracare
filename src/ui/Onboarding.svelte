<script lang="ts">
  import { createVault } from '../lib/session.svelte';
  import { DEFAULT_SETTINGS, isValidWindow, type TimeWindow } from '../core/settings';

  const MIN_LENGTH = 10;
  const TOTAL = 5;

  let step = $state(0);
  let passphrase = $state('');
  let confirm = $state('');
  let understood = $state(false);
  let windows = $state<TimeWindow[]>([]);
  let busy = $state(false);
  let error = $state('');

  const passOk = $derived(passphrase.length >= MIN_LENGTH && passphrase === confirm && understood);
  const windowsOk = $derived(windows.every(isValidWindow));

  function addWindow() {
    windows.push({ start: '09:00', end: '10:00' });
  }

  async function finish() {
    busy = true;
    error = '';
    try {
      await createVault(passphrase, { ...DEFAULT_SETTINGS, checkinWindows: $state.snapshot(windows) });
    } catch (e) {
      error = 'Qualcosa non è andato. Riprova tra un momento.';
      console.error(e);
      busy = false;
    }
  }
</script>

<div class="dots" aria-hidden="true">
  {#each Array(TOTAL) as _, i}<span class:on={i === step}></span>{/each}
</div>

{#if step === 0}
  <section class="fade" aria-labelledby="s0">
    <h1 id="s0">Ti do il benvenuto in Kuracare</h1>
    <p>Un posto tranquillo per prenderti cura di te: le cose da fare, come stai, i gesti che ti fanno bene.</p>
    <p>Kuracare è un diario personale. Registra e riassume, ma <strong>non dà consigli sanitari</strong>: per quelli c'è chi ti segue.</p>
    <div class="actions"><button class="primary" onclick={() => (step = 1)}>Avanti</button></div>
  </section>
{:else if step === 1}
  <section class="fade" aria-labelledby="s1">
    <h1 id="s1">Tutto resta qui</h1>
    <p><span class="badge-local">Solo su questo telefono</span></p>
    <p>Quello che scrivi resta su questo dispositivo, cifrato. Niente account, niente cloud, nessuno che guarda.</p>
    <p>I dati escono solo se lo decidi tu, con un'esportazione. Ogni volta Kuracare se lo annota, così sai sempre cosa è uscito e quando.</p>
    <div class="actions">
      <button class="primary" onclick={() => (step = 2)}>Avanti</button>
      <button class="quiet" onclick={() => (step = 0)}>Indietro</button>
    </div>
  </section>
{:else if step === 2}
  <section class="fade" aria-labelledby="s2">
    <h1 id="s2">Una chiave solo tua</h1>
    <p>Ora sceglierai una <strong>passphrase</strong>: una frase che conosci solo tu. Serve a cifrare i dati e ad aprire l'app.</p>
    <p><strong>Se la dimentichi, i dati non si possono recuperare</strong>, da nessuno: nemmeno da chi ha scritto l'app. È il prezzo della riservatezza.</p>
    <p class="muted">Più avanti potrai fare un backup cifrato, da tenere in un posto sicuro.</p>
    <div class="actions">
      <button class="primary" onclick={() => (step = 3)}>Scelgo la passphrase</button>
      <button class="quiet" onclick={() => (step = 1)}>Indietro</button>
    </div>
  </section>
{:else if step === 3}
  <section class="fade" aria-labelledby="s3">
    <h1 id="s3">La tua passphrase</h1>
    <p class="muted">Qualche parola che per te ha senso funziona meglio di una password complicata, per esempio quattro parole a caso.</p>
    <form onsubmit={(e) => { e.preventDefault(); if (passOk) step = 4; }}>
      <label for="pp">Passphrase</label>
      <input id="pp" type="password" autocomplete="new-password" bind:value={passphrase} />
      <p class="hint">Almeno {MIN_LENGTH} caratteri{passphrase.length > 0 && passphrase.length < MIN_LENGTH ? ` (ne mancano ${MIN_LENGTH - passphrase.length})` : ''}.</p>
      <label for="pp2">Scrivila di nuovo</label>
      <input id="pp2" type="password" autocomplete="new-password" bind:value={confirm} />
      {#if confirm.length > 0 && confirm !== passphrase}<p class="hint">Le due frasi non coincidono ancora.</p>{/if}
      <label class="check">
        <input type="checkbox" bind:checked={understood} />
        <span>Ho capito: se dimentico la passphrase, i dati non si possono recuperare.</span>
      </label>
      <div class="actions">
        <button class="primary" type="submit" disabled={!passOk}>Avanti</button>
        <button class="quiet" type="button" onclick={() => (step = 2)}>Indietro</button>
      </div>
    </form>
  </section>
{:else}
  <section class="fade" aria-labelledby="s4">
    <h1 id="s4">Quando ti va di sentirti chiedere come stai?</h1>
    <p>Puoi scegliere uno o più momenti della giornata. Saranno inviti, mai obblighi, e potrai cambiarli quando vuoi.</p>
    {#each windows as w, i}
      <div class="card row">
        <label class="sr" for={`ws${i}`}>Dalle</label>
        <input id={`ws${i}`} type="time" bind:value={w.start} />
        <label class="sr" for={`we${i}`}>alle</label>
        <input id={`we${i}`} type="time" bind:value={w.end} />
        <button class="quiet" onclick={() => windows.splice(i, 1)} aria-label="Togli questo momento">Togli</button>
      </div>
    {/each}
    {#if !windowsOk}<p class="hint">Controlla gli orari: l'inizio deve venire prima della fine.</p>{/if}
    <button onclick={addWindow}>Aggiungi un momento</button>
    {#if error}<p class="error" role="alert">{error}</p>{/if}
    <div class="actions">
      <button class="primary" disabled={busy || !windowsOk} onclick={finish}>
        {busy ? 'Sto preparando la chiave…' : windows.length ? 'Iniziamo' : 'Per ora nessuno, iniziamo'}
      </button>
      <button class="quiet" disabled={busy} onclick={() => (step = 3)}>Indietro</button>
    </div>
  </section>
{/if}

<style>
  .sr { margin: 0; font-weight: 400; }
</style>
