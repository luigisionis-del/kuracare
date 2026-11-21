<script lang="ts">
  import { diary } from '../../lib/diary.svelte';
  import { go } from '../../lib/nav.svelte';

  function follow() {
    const id = diary.toast?.action?.eventId;
    if (!id) return;
    go({ name: 'edit', id });
    diary.toast = null;
  }
</script>

{#if diary.toast}
  <div class="toast fade" role="status">
    <span>{diary.toast.text}</span>
    {#if diary.toast.action}
      <button class="small" onclick={follow}>{diary.toast.action.label}</button>
    {/if}
  </div>
{/if}
