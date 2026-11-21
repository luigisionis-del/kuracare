<script lang="ts">
  import { unlock, destroyEverything } from '../lib/session.svelte';
  import { WrongPassphraseError } from '../core/crypto/keyring';

  const CONFIRM_WORD = 'CANCELLA';

  let passphrase = $state('');
  let busy = $state(false);
  let error = $state('');
  let forgot = $state(false);
  let confirmText = $state('');

  async function submit(e: SubmitEvent) {
    e.preventDefault();
    if (!passphrase) return;
    busy = true;
    error = '';
    try {
      await unlock(passphrase);
    } catch (err) {
      error = err instanceof WrongPassphraseError
        ? 'Non sembra la passphrase giusta. Riprova con calma.'
        : 'Qualcosa non è andato. Riprova tra un momento.';
      if (!(err instanceof WrongPassphraseError)) console.error(err);
    } finally {
      busy = false;
      passphrase = '';
    }
  }
</script>

{#if !forgot}
  <section class="fade" aria-labelledby="u">
    <h1 id="u">Che bello rivederti</h1>
    <p class="muted">Kuracare è chiusa per proteggere i tuoi dati.</p>
    <form onsubmit={submit}>
      <label for="pp">Passphrase</label>
      <input id="pp" type="password" autocomplete="current-password" bind:value={passphrase} />
      {#if error}<p class="error" role="alert">{error}</p>{/if}
      <div class="actions">
        <button class="primary" type="submit" disabled={busy || !passphrase}>{busy ? 'Apro…' : 'Apri'}</button>
        <button class="quiet" type="button" onclick={() => (forgot = true)}>Ho dimenticato la passphrase</button>
      </div>
    </form>
  </section>
{:else}
  <section class="fade" aria-labelledby="f">
    <h1 id="f">Passphrase dimenticata</h1>
    <p>Mi dispiace: senza passphrase i dati non si possono aprire, da nessuno. È ciò che li tiene al sicuro.</p>
    <p>Se hai un backup e ricordi la passphrase con cui l'hai fatto, potrai ripartire da lì.</p>
    <p>Altrimenti puoi cancellare tutto e ricominciare da capo. <strong>Non si può annullare.</strong></p>
    <label for="c">Per cancellare tutto scrivi {CONFIRM_WORD}</label>
    <input id="c" type="text" autocomplete="off" bind:value={confirmText} />
    <div class="actions">
      <button class="danger" disabled={confirmText !== CONFIRM_WORD} onclick={destroyEverything}>Cancella tutto e ricomincia</button>
      <button class="quiet" onclick={() => { forgot = false; confirmText = ''; }}>Torna indietro</button>
    </div>
  </section>
{/if}
