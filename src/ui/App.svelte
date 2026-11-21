<script lang="ts">
  import { onMount } from 'svelte';
  import { boot, session } from '../lib/session.svelte';
  import { nav } from '../lib/nav.svelte';
  import Onboarding from './Onboarding.svelte';
  import Unlock from './Unlock.svelte';
  import Home from './Home.svelte';
  import Checkin from './views/Checkin.svelte';
  import Thought from './views/Thought.svelte';
  import Meal from './views/Meal.svelte';
  import Edit from './views/Edit.svelte';
  import Enrich from './views/Enrich.svelte';
  import Lists from './views/Lists.svelte';
  import Toast from './parts/Toast.svelte';
  import { applyUpdate, update } from '../lib/update.svelte';

  onMount(() => {
    boot();
  });
</script>

<main>
  {#if update.ready}
    <div class="card soft update" role="status">
      <span>C'è una versione nuova di Kuracare.</span>
      <button class="small" onclick={applyUpdate}>Aggiorna</button>
    </div>
  {/if}
  {#if session.phase === 'loading'}
    <p class="muted">Un attimo…</p>
  {:else if session.phase === 'onboarding'}
    <Onboarding />
  {:else if session.phase === 'locked'}
    <Unlock />
  {:else}
    {#if nav.view.name === 'home'}
      <Home />
    {:else if nav.view.name === 'checkin'}
      <Checkin />
    {:else if nav.view.name === 'thought'}
      <Thought />
    {:else if nav.view.name === 'meal'}
      <Meal mealType={nav.view.mealType} />
    {:else if nav.view.name === 'edit'}
      {#key nav.view.id}<Edit id={nav.view.id} />{/key}
    {:else if nav.view.name === 'enrich'}
      <Enrich />
    {:else if nav.view.name === 'lists'}
      <Lists />
    {/if}
    <Toast />
  {/if}
</main>
