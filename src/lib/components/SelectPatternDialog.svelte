<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';

  const dispatch = createEventDispatcher<{ select: { pattern: string }; cancel: void }>();

  let pattern = '*';
  let inputEl: HTMLInputElement;

  onMount(() => {
    inputEl?.focus();
    inputEl?.select();
  });

  function confirm() {
    if (pattern.trim()) dispatch('select', { pattern: pattern.trim() });
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') { e.stopPropagation(); dispatch('cancel'); }
    if (e.key === 'Enter') { e.stopPropagation(); confirm(); }
  }
</script>

<!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
<div class="overlay" on:click|self={() => dispatch('cancel')}>
  <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
  <div class="dialog" role="dialog" tabindex="-1" on:keydown={handleKeydown}>
    <div class="dialog-title">SELECT BY PATTERN</div>
    <div class="dialog-body">
      <label for="pattern-input">Pattern:</label>
      <input id="pattern-input" bind:this={inputEl} bind:value={pattern} />
    </div>
    <div class="dialog-footer">
      <button on:click={confirm}>Select</button>
      <button on:click={() => dispatch('cancel')}>Cancel (Esc)</button>
    </div>
  </div>
</div>
