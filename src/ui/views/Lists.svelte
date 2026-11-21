<script lang="ts">
  import { ROUTES, positive } from '../../core/events/kinds';
  import { newId, type Lists } from '../../core/events/lists';
  import { ROUTE_LABELS } from '../../lib/labels';
  import { diary, setLists, showToast } from '../../lib/diary.svelte';
  import { back } from '../../lib/nav.svelte';

  const show = (n: number | null) => (n === null ? '' : String(n).replace('.', ','));

  const snap = $state.snapshot(diary.lists) as Lists;
  let strains = $state(snap.strains.map((s) => ({ ...s, amountText: show(s.amount), thcText: show(s.thcPct), cbdText: show(s.cbdPct) })));
  let supplements = $state(snap.supplements.map((s) => ({ ...s, amountText: show(s.amount) })));
  let careLabels = $state([...snap.careLabels]);
  let thoughtLabels = $state([...snap.thoughtLabels]);
  let newCare = $state('');
  let newThought = $state('');
  let busy = $state(false);

  function addStrain(group: 'thc' | 'cbd') {
    strains.push({ id: newId('s'), name: '', group, thcPct: null, cbdPct: null, amount: null, unit: 'g', route: null, amountText: '', thcText: '', cbdText: '' });
  }
  function addSupplement() {
    supplements.push({ id: newId('i'), name: '', amount: null, unit: 'mg', amountText: '' });
  }
  function addLabel(list: string[], text: string): boolean {
    const t = text.trim();
    if (!t || list.includes(t)) return false;
    list.push(t);
    return true;
  }

  async function save() {
    busy = true;
    const lists: Lists = {
      version: 1,
      strains: strains.filter((s) => s.name.trim()).map(({ amountText, thcText, cbdText, ...s }) => ({
        ...s, name: s.name.trim(), amount: positive(amountText), thcPct: positive(thcText, 100), cbdPct: positive(cbdText, 100),
      })),
      supplements: supplements.filter((s) => s.name.trim()).map(({ amountText, ...s }) => ({ ...s, name: s.name.trim(), amount: positive(amountText) })),
      careLabels: [...careLabels],
      thoughtLabels: [...thoughtLabels],
    };
    await setLists(lists);
    showToast({ text: 'Liste salvate.' });
    back();
  }
</script>

<section class="fade" aria-labelledby="ls">
  <h1 id="ls">Le tue liste</h1>
  <p class="muted">Sono i bottoni da un tocco. I valori li scegli tu: Kuracare non suggerisce varietà né quantità.</p>

  <h2>Varietà</h2>
  {#each strains as s, i (s.id)}
    <div class="card" role="group" aria-label={s.name || 'Nuova varietà'}>
      <label for={`sn${i}`}>Nome</label>
      <input id={`sn${i}`} type="text" bind:value={s.name} />
      <div class="chips" role="group" aria-label="Prevalenza">
        <button class="chip" aria-pressed={s.group === 'thc'} onclick={() => (s.group = 'thc')}>Prevalenza THC</button>
        <button class="chip" aria-pressed={s.group === 'cbd'} onclick={() => (s.group = 'cbd')}>Prevalenza CBD</button>
      </div>
      <div class="grid2">
        <div><label for={`st${i}`}>THC %</label><input id={`st${i}`} type="text" inputmode="decimal" bind:value={s.thcText} /></div>
        <div><label for={`sc${i}`}>CBD %</label><input id={`sc${i}`} type="text" inputmode="decimal" bind:value={s.cbdText} /></div>
        <div><label for={`sa${i}`}>Quantità abituale</label><input id={`sa${i}`} type="text" inputmode="decimal" bind:value={s.amountText} /></div>
        <div><label for={`su${i}`}>Unità</label><input id={`su${i}`} type="text" bind:value={s.unit} /></div>
      </div>
      <p class="label">Via abituale</p>
      <div class="chips">
        {#each ROUTES as r}
          <button class="chip" aria-pressed={s.route === r} onclick={() => (s.route = s.route === r ? null : r)}>{ROUTE_LABELS[r]}</button>
        {/each}
      </div>
      <button class="quiet small" onclick={() => strains.splice(i, 1)}>Togli dalla lista</button>
    </div>
  {/each}
  <div class="row">
    <button class="small" onclick={() => addStrain('thc')}>+ Varietà THC</button>
    <button class="small" onclick={() => addStrain('cbd')}>+ Varietà CBD</button>
  </div>

  <h2>Integratori</h2>
  {#each supplements as s, i (s.id)}
    <div class="card" role="group" aria-label={s.name || 'Nuovo integratore'}>
      <label for={`in${i}`}>Nome</label>
      <input id={`in${i}`} type="text" bind:value={s.name} />
      <div class="grid2">
        <div><label for={`ia${i}`}>Quantità abituale</label><input id={`ia${i}`} type="text" inputmode="decimal" bind:value={s.amountText} /></div>
        <div><label for={`iu${i}`}>Unità</label><input id={`iu${i}`} type="text" bind:value={s.unit} /></div>
      </div>
      <button class="quiet small" onclick={() => supplements.splice(i, 1)}>Togli dalla lista</button>
    </div>
  {/each}
  <button class="small" onclick={addSupplement}>+ Integratore</button>

  <h2>Gesti di cura</h2>
  <div class="chips">
    {#each careLabels as l, i}<button class="chip" aria-label={`Togli ${l}`} onclick={() => careLabels.splice(i, 1)}>{l} ×</button>{/each}
  </div>
  <form class="row" onsubmit={(e) => { e.preventDefault(); if (addLabel(careLabels, newCare)) newCare = ''; }}>
    <input type="text" aria-label="Nuovo gesto di cura" placeholder="Un nuovo gesto" bind:value={newCare} />
    <button class="small" type="submit">Aggiungi</button>
  </form>

  <h2>Pensieri che tornano</h2>
  <div class="chips">
    {#each thoughtLabels as l, i}<button class="chip" aria-label={`Togli ${l}`} onclick={() => thoughtLabels.splice(i, 1)}>{l} ×</button>{/each}
  </div>
  <form class="row" onsubmit={(e) => { e.preventDefault(); if (addLabel(thoughtLabels, newThought)) newThought = ''; }}>
    <input type="text" aria-label="Nuova etichetta per i pensieri" placeholder="Una nuova etichetta" bind:value={newThought} />
    <button class="small" type="submit">Aggiungi</button>
  </form>

  <div class="actions">
    <button class="primary" disabled={busy} onclick={save}>Salva le liste</button>
    <button class="quiet" onclick={back}>Lascia com'erano</button>
  </div>
</section>
