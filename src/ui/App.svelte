<script lang="ts">
  import { onMount } from 'svelte';
  import { boot, session } from '../lib/session.svelte';
  import Onboarding from './Onboarding.svelte';
  import Unlock from './Unlock.svelte';
  import Home from './Home.svelte';

  onMount(() => {
    boot();
  });
</script>

<main>
  {#if session.phase === 'loading'}
    <p class="muted">Un attimo…</p>
  {:else if session.phase === 'onboarding'}
    <Onboarding />
  {:else if session.phase === 'locked'}
    <Unlock />
  {:else}
    <Home />
  {/if}
</main>
