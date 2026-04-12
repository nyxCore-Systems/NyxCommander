<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { colorRulesStore, type ColorRule } from '$lib/stores/colorRulesStore';

  const dispatch = createEventDispatcher<{ close: void }>();

  $: rules = $colorRulesStore;

  function handleKey(e: KeyboardEvent) {
    e.stopPropagation();
    if (e.key === 'Escape') dispatch('close');
  }
</script>

<!-- svelte-ignore a11y-no-static-element-interactions -->
<div class="overlay" on:keydown={handleKey}>
  <div class="dialog">
    <div class="titlebar">
      <span class="title">Color Rules</span>
      <div class="actions">
        <button on:click={() => colorRulesStore.addRule()}>+ Add rule</button>
        <button class="close-btn" on:click={() => dispatch('close')}>✕</button>
      </div>
    </div>

    <div class="rules-list">
      {#each rules as rule, i (rule.id)}
        <div class="rule-row">
          <input
            type="checkbox"
            checked={rule.enabled}
            on:change={e => colorRulesStore.updateRule(rule.id, { enabled: (e.target as HTMLInputElement).checked })}
          />

          <div class="rule-body">
            <div class="rule-top">
              <input
                class="lbl"
                value={rule.label}
                on:input={e => colorRulesStore.updateRule(rule.id, { label: (e.target as HTMLInputElement).value })}
                placeholder="Label"
              />
              <div class="color-chips">
                <label class="color-lbl">FG
                  <input
                    type="color"
                    value={rule.fg || '#ffffff'}
                    on:input={e => colorRulesStore.updateRule(rule.id, { fg: (e.target as HTMLInputElement).value })}
                  />
                </label>
                <label class="color-lbl">BG
                  <input
                    type="color"
                    value={rule.bg || '#000000'}
                    on:input={e => colorRulesStore.updateRule(rule.id, { bg: (e.target as HTMLInputElement).value })}
                  />
                  <button
                    class="clear-btn"
                    on:click={() => colorRulesStore.updateRule(rule.id, { bg: '' })}
                    title="Clear background"
                  >✕</button>
                </label>
                <label class="check"><input type="checkbox" checked={rule.bold} on:change={e => colorRulesStore.updateRule(rule.id, { bold: (e.target as HTMLInputElement).checked })} /> Bold</label>
                <label class="check"><input type="checkbox" checked={rule.italic} on:change={e => colorRulesStore.updateRule(rule.id, { italic: (e.target as HTMLInputElement).checked })} /> Italic</label>
              </div>
            </div>
            <div class="rule-fields">
              <label>
                <span>Extensions</span>
                <input
                  value={rule.matchExtensions}
                  on:input={e => colorRulesStore.updateRule(rule.id, { matchExtensions: (e.target as HTMLInputElement).value })}
                  placeholder="rs,toml,json…"
                />
              </label>
              <label>
                <span>Name contains</span>
                <input
                  value={rule.matchNamePattern}
                  on:input={e => colorRulesStore.updateRule(rule.id, { matchNamePattern: (e.target as HTMLInputElement).value })}
                  placeholder="Makefile,README…"
                />
              </label>
              <label class="size-lbl">
                <span>Min size</span>
                <input
                  type="number"
                  value={rule.matchMinSize}
                  on:input={e => colorRulesStore.updateRule(rule.id, { matchMinSize: Number((e.target as HTMLInputElement).value) })}
                  min="0"
                />
              </label>
            </div>
          </div>

          <div class="rule-actions">
            {#if i > 0}
              <button class="icon-btn" on:click={() => colorRulesStore.reorder(i, i - 1)}>↑</button>
            {/if}
            {#if i < rules.length - 1}
              <button class="icon-btn" on:click={() => colorRulesStore.reorder(i, i + 1)}>↓</button>
            {/if}
            <button class="icon-btn del" on:click={() => colorRulesStore.removeRule(rule.id)}>✕</button>
          </div>

          <!-- preview swatch -->
          <span
            class="preview-swatch"
            style="color:{rule.fg || 'inherit'}; background:{rule.bg||'transparent'}; font-weight:{rule.bold?'bold':'normal'}; font-style:{rule.italic?'italic':'normal'}"
          >Abc</span>
        </div>
      {/each}
      {#if rules.length === 0}
        <div class="empty">No rules — click "+ Add rule"</div>
      {/if}
    </div>
  </div>
</div>

<style>
  .overlay {
    position: fixed; inset: 0; background: rgba(0,0,0,0.75);
    display: flex; align-items: center; justify-content: center; z-index: 100;
  }
  .dialog {
    background: var(--bg-panel);
    border: 1px solid var(--border-panel);
    width: min(700px, 95vw);
    max-height: 80vh;
    display: flex;
    flex-direction: column;
    font-family: 'Courier New', monospace;
    font-size: 12px;
  }
  .titlebar {
    display: flex; align-items: center; justify-content: space-between;
    padding: 6px 10px; background: var(--bg-header); border-bottom: 1px solid var(--border-dim);
  }
  .title { color: var(--accent); font-size: 11px; text-transform: uppercase; letter-spacing: 0.06em; }
  .actions { display: flex; gap: 8px; align-items: center; }
  .close-btn {
    background: none; border: none; color: var(--text-dim); cursor: pointer; font-size: 14px;
  }
  .close-btn:hover { color: var(--danger); }
  button {
    background: var(--bg); border: 1px solid var(--border-dim); color: var(--text-dim);
    font-family: inherit; font-size: 10px; padding: 3px 8px; cursor: pointer;
  }
  button:hover { border-color: var(--accent); color: var(--accent); }

  .rules-list { flex: 1; overflow-y: auto; scrollbar-width: thin; scrollbar-color: var(--scrollbar-thumb) var(--bg); }
  .empty { padding: 20px; text-align: center; color: var(--text-dim); }

  .rule-row {
    display: flex; align-items: flex-start; gap: 8px;
    padding: 8px 10px; border-bottom: 1px solid var(--border-dim);
  }
  .rule-row:hover { background: var(--bg-header); }
  .rule-row input[type="checkbox"] { margin-top: 4px; flex-shrink: 0; }

  .rule-body { flex: 1; display: flex; flex-direction: column; gap: 6px; min-width: 0; }
  .rule-top { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }

  input.lbl {
    background: var(--bg); border: 1px solid var(--border-dim); color: var(--text);
    font-family: inherit; font-size: 12px; padding: 2px 6px; width: 120px;
  }

  .color-chips { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }
  .color-lbl {
    display: flex; align-items: center; gap: 3px;
    color: var(--text-dim); font-size: 10px; cursor: default;
  }
  .color-lbl input[type="color"] { width: 24px; height: 18px; border: none; cursor: pointer; }
  .clear-btn { padding: 1px 4px; font-size: 9px; }
  .check { display: flex; align-items: center; gap: 3px; color: var(--text-dim); font-size: 10px; }

  .rule-fields { display: flex; gap: 8px; flex-wrap: wrap; }
  .rule-fields label {
    display: flex; align-items: center; gap: 4px; flex: 1; min-width: 140px;
    color: var(--text-dim); font-size: 10px;
  }
  .rule-fields label span { white-space: nowrap; }
  .rule-fields input {
    flex: 1; background: var(--bg); border: 1px solid var(--border-dim); color: var(--text);
    font-family: inherit; font-size: 11px; padding: 2px 5px;
  }
  label.size-lbl input { width: 80px; }

  .rule-actions { display: flex; flex-direction: column; gap: 3px; flex-shrink: 0; }
  .icon-btn { padding: 1px 5px; font-size: 11px; }
  .icon-btn.del:hover { border-color: var(--danger); color: var(--danger); }

  .preview-swatch {
    flex-shrink: 0; font-family: 'Courier New', monospace; font-size: 12px;
    padding: 2px 6px; border: 1px solid var(--border-dim); align-self: center;
  }
</style>
