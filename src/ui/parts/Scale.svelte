<script lang="ts">
  /** 0-10 answer that starts unanswered and can always be cleared. */
  let { id, label, low, high, value = $bindable(null) }: {
    id: string; label: string; low: string; high: string; value: number | null;
  } = $props();

  function set(e: Event) {
    value = Number((e.currentTarget as HTMLInputElement).value);
  }
</script>

<div class="scale">
  <div class="scale-head">
    <label for={id}>{label}</label>
    <span class="scale-val" aria-hidden="true">{value ?? '–'}</span>
    {#if value !== null}
      <button type="button" class="quiet small" onclick={() => (value = null)} aria-label={`Togli la risposta: ${label}`}>togli</button>
    {/if}
  </div>
  <input
    {id}
    type="range" min="0" max="10" step="1"
    class:unset={value === null}
    value={value ?? 5}
    oninput={set}
    onclick={set}
    aria-valuetext={value === null ? 'non indicato' : String(value)}
  />
  <div class="scale-ends" aria-hidden="true"><span>0 · {low}</span><span>{high} · 10</span></div>
</div>
