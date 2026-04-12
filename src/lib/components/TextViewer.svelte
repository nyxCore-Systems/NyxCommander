<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { marked } from 'marked';
  import hljs from 'highlight.js/lib/core';

  // Register only the grammars we need — keeps the bundle small
  import langBash       from 'highlight.js/lib/languages/bash';
  import langC          from 'highlight.js/lib/languages/c';
  import langCpp        from 'highlight.js/lib/languages/cpp';
  import langCss        from 'highlight.js/lib/languages/css';
  import langDiff       from 'highlight.js/lib/languages/diff';
  import langGo         from 'highlight.js/lib/languages/go';
  import langIni        from 'highlight.js/lib/languages/ini';
  import langJava       from 'highlight.js/lib/languages/java';
  import langJs         from 'highlight.js/lib/languages/javascript';
  import langJson       from 'highlight.js/lib/languages/json';
  import langKotlin     from 'highlight.js/lib/languages/kotlin';
  import langLua        from 'highlight.js/lib/languages/lua';
  import langMarkdown   from 'highlight.js/lib/languages/markdown';
  import langPython     from 'highlight.js/lib/languages/python';
  import langRust       from 'highlight.js/lib/languages/rust';
  import langShell      from 'highlight.js/lib/languages/shell';
  import langSql        from 'highlight.js/lib/languages/sql';
  import langSwift      from 'highlight.js/lib/languages/swift';
  import langTs         from 'highlight.js/lib/languages/typescript';
  import langXml        from 'highlight.js/lib/languages/xml';
  import langYaml       from 'highlight.js/lib/languages/yaml';

  hljs.registerLanguage('bash',       langBash);
  hljs.registerLanguage('c',          langC);
  hljs.registerLanguage('cpp',        langCpp);
  hljs.registerLanguage('css',        langCss);
  hljs.registerLanguage('diff',       langDiff);
  hljs.registerLanguage('go',         langGo);
  hljs.registerLanguage('ini',        langIni);
  hljs.registerLanguage('java',       langJava);
  hljs.registerLanguage('javascript', langJs);
  hljs.registerLanguage('json',       langJson);
  hljs.registerLanguage('kotlin',     langKotlin);
  hljs.registerLanguage('lua',        langLua);
  hljs.registerLanguage('markdown',   langMarkdown);
  hljs.registerLanguage('python',     langPython);
  hljs.registerLanguage('rust',       langRust);
  hljs.registerLanguage('shell',      langShell);
  hljs.registerLanguage('sql',        langSql);
  hljs.registerLanguage('swift',      langSwift);
  hljs.registerLanguage('typescript', langTs);
  hljs.registerLanguage('xml',        langXml);
  hljs.registerLanguage('yaml',       langYaml);

  // Extension → hljs language alias
  const EXT_LANG: Record<string, string> = {
    bash: 'bash', sh: 'bash', zsh: 'bash',
    c: 'c', h: 'c',
    cc: 'cpp', cpp: 'cpp', cxx: 'cpp', hpp: 'cpp',
    css: 'css',
    diff: 'diff', patch: 'diff',
    go: 'go',
    ini: 'ini', cfg: 'ini', conf: 'ini',
    java: 'java',
    js: 'javascript', mjs: 'javascript', cjs: 'javascript',
    jsx: 'javascript',
    json: 'json', jsonc: 'json',
    kt: 'kotlin', kts: 'kotlin',
    lua: 'lua',
    md: 'markdown',
    py: 'python', pyw: 'python',
    rs: 'rust',
    sql: 'sql',
    swift: 'swift',
    toml: 'ini',
    ts: 'typescript', tsx: 'typescript',
    // Svelte: no native grammar — fall back to xml for tag highlighting
    svelte: 'xml',
    html: 'xml', htm: 'xml', xml: 'xml', svg: 'xml',
    yml: 'yaml', yaml: 'yaml',
  };

  export let path: string;

  const dispatch = createEventDispatcher<{ close: void }>();

  let content = '';
  let loading = true;
  let error = '';
  let lines: string[] = [];
  let highlightedHtml = '';
  let renderedHtml = '';
  let overlayEl: HTMLElement;
  let codeBodyEl: HTMLElement;
  let mdBodyEl: HTMLElement;

  const filename = path.split('/').pop() ?? path;
  const ext = filename.includes('.') ? filename.split('.').pop()!.toLowerCase() : '';
  const lang = EXT_LANG[ext] ?? '';

  // Markdown files get rendered view by default; toggle available
  const isMarkdown = ext === 'md' || ext === 'mdx';
  let renderMode: 'rendered' | 'source' = isMarkdown ? 'rendered' : 'source';

  marked.setOptions({ breaks: true, gfm: true });

  onMount(async () => {
    overlayEl.focus();
    try {
      content = await invoke<string>('read_text_file', { path });
      lines = content.split('\n');
      if (lines.at(-1) === '') lines = lines.slice(0, -1);

      if (isMarkdown) {
        renderedHtml = marked.parse(content) as string;
      }

      if (lang) {
        highlightedHtml = hljs.highlight(content, { language: lang }).value;
      } else {
        highlightedHtml = content
          .replace(/&/g, '&amp;')
          .replace(/</g, '&lt;')
          .replace(/>/g, '&gt;');
      }
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  });

  const LINE_PX = 18; // 12px font × 1.5 line-height

  function handleKey(e: KeyboardEvent) {
    switch (e.key) {
      case 'Escape':   dispatch('close'); break;
      case 'F4':       openExternal(); break;
      case 'ArrowDown': scroll(LINE_PX);  e.preventDefault(); break;
      case 'ArrowUp':   scroll(-LINE_PX); e.preventDefault(); break;
      case 'PageDown':  scroll(pageSize()); e.preventDefault(); break;
      case 'PageUp':    scroll(-pageSize()); e.preventDefault(); break;
      case 'Home':      if (e.ctrlKey || e.metaKey) scrollTo(0); e.preventDefault(); break;
      case 'End':       if (e.ctrlKey || e.metaKey) scrollTo(Infinity); e.preventDefault(); break;
    }
  }

  function scrollEl() {
    return renderMode === 'rendered' ? mdBodyEl : codeBodyEl;
  }

  function scroll(delta: number) {
    const el = scrollEl();
    if (el) el.scrollTop += delta;
  }

  function scrollTo(pos: number) {
    const el = scrollEl();
    if (el) el.scrollTop = pos;
  }

  function pageSize() {
    const el = scrollEl();
    return el ? el.clientHeight * 0.9 : 400;
  }

  function openExternal() {
    invoke('open_file', { path }).catch(console.error);
  }

  $: lineCount = lines.length;
</script>

<!-- svelte-ignore a11y-no-static-element-interactions -->
<div class="overlay" bind:this={overlayEl} on:keydown={handleKey} tabindex="-1">
  <div class="viewer" role="dialog" tabindex="-1">

    <div class="title-bar">
      <span class="filename">{filename}</span>
      {#if ext}<span class="ext-badge">{renderMode === 'rendered' ? 'rendered' : (lang || ext)}</span>{/if}
      <span class="path-dim">{path}</span>
      {#if isMarkdown}
        <button
          class="hdr-btn toggle-btn"
          class:active-mode={renderMode === 'rendered'}
          on:click={() => renderMode = renderMode === 'rendered' ? 'source' : 'rendered'}
          title="Toggle rendered / source view"
        >
          {renderMode === 'rendered' ? '⟨/⟩ Source' : '⬚ Rendered'}
        </button>
      {/if}
      <button class="hdr-btn" on:click={openExternal} title="Open in system editor (F4)">Open in editor</button>
      <button class="close-btn" on:click={() => dispatch('close')}>✕</button>
    </div>

    {#if loading}
      <div class="status-row">Loading…</div>
    {:else if error}
      <div class="status-row error">{error}</div>
    {:else}
      <div class="info-bar">
        <span>{lineCount} line{lineCount !== 1 ? 's' : ''}</span>
        <span>{content.length} chars</span>
        {#if lang && renderMode === 'source'}<span class="lang-tag">{lang}</span>{/if}
        <span class="hint">↑↓ scroll · PgUp/Dn page · Esc close · F4 open in editor</span>
      </div>

      {#if renderMode === 'rendered' && isMarkdown}
        <!-- Rendered markdown view -->
        <div class="md-body" bind:this={mdBodyEl}>
          {@html renderedHtml}
        </div>
      {:else}
        <!-- Syntax-highlighted source view -->
        <div class="code-area">
          <div class="gutter">
            {#each lines as _, i}
              <span class="line-no">{i + 1}</span>
            {/each}
          </div>
          <pre class="code-body" bind:this={codeBodyEl}><code>{@html highlightedHtml}</code></pre>
        </div>
      {/if}
    {/if}

  </div>
</div>

<style>
  .overlay {
    position: fixed; inset: 0;
    background: rgba(0,0,0,0.88);
    display: flex; align-items: center; justify-content: center;
    z-index: 100;
  }

  .viewer {
    display: flex; flex-direction: column;
    width: 900px; max-width: 96vw; height: 82vh;
    background: var(--bg);
    border: 1px solid var(--border-panel);
    font-family: 'Courier New', monospace;
  }

  .title-bar {
    display: flex; align-items: center; gap: 8px;
    padding: 0 10px; height: 26px; flex-shrink: 0;
    background: var(--bg-header);
    border-bottom: 1px solid var(--border-panel);
    font-size: 11px; overflow: hidden;
  }
  .filename {
    color: var(--border-panel); font-weight: bold;
    white-space: nowrap;
  }
  .ext-badge {
    background: var(--bg-fnkey); color: var(--text-dim);
    border: 1px solid var(--border-dim);
    font-size: 9px; padding: 0 4px; flex-shrink: 0;
    text-transform: uppercase;
  }
  .path-dim {
    color: var(--text-dim); font-size: 10px; flex: 1;
    overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
  }
  .hdr-btn {
    background: transparent; border: 1px solid var(--border-dim);
    color: var(--text-dim); font-family: 'Courier New', monospace;
    font-size: 10px; padding: 1px 8px; cursor: pointer; flex-shrink: 0;
  }
  .hdr-btn:hover { border-color: var(--border-panel); color: var(--border-panel); }
  .toggle-btn.active-mode { border-color: var(--accent); color: var(--accent); }

  .close-btn {
    background: none; border: none; color: var(--text-dim);
    cursor: pointer; font-size: 13px; padding: 0 2px; flex-shrink: 0;
  }
  .close-btn:hover { color: var(--text-danger); }

  .status-row {
    padding: 20px 16px; color: var(--text-dim); font-size: 12px;
    font-family: 'Courier New', monospace;
  }
  .status-row.error { color: var(--text-danger); }

  .info-bar {
    display: flex; gap: 16px; align-items: center;
    padding: 0 10px; height: 20px; flex-shrink: 0;
    background: var(--bg-panel);
    border-bottom: 1px solid var(--border-dim);
    font-size: 10px; color: var(--text-dim);
  }
  .lang-tag { color: var(--border-panel); }
  .hint { margin-left: auto; }

  .code-area {
    flex: 1; display: flex; overflow: hidden;
  }

  .gutter {
    display: flex; flex-direction: column;
    padding: 6px 0;
    background: var(--bg-panel);
    border-right: 1px solid var(--border-dim);
    overflow: hidden; flex-shrink: 0;
    user-select: none;
  }

  .line-no {
    display: block;
    padding: 0 8px;
    color: var(--text-dim); font-size: 11px;
    font-family: 'Courier New', monospace;
    line-height: 1.5;
    text-align: right; white-space: nowrap;
  }

  .code-body {
    flex: 1;
    margin: 0; padding: 6px 12px;
    overflow: auto;
    font-family: 'Courier New', monospace;
    font-size: 12px; line-height: 1.5;
    tab-size: 4;
    white-space: pre;
    scrollbar-width: thin;
    scrollbar-color: var(--scrollbar-thumb) var(--bg);
  }

  .code-body code {
    display: block;
    background: transparent;
    font-family: inherit;
    font-size: inherit;
    line-height: inherit;
    white-space: pre;
  }

  /* ── Nyx syntax theme ─────────────────────────────────────────────────── */
  /* Plain text */
  .code-body :global(.hljs)          { color: #e0e0f0; }

  /* Keywords: if, for, fn, return, pub, use, import … */
  .code-body :global(.hljs-keyword),
  .code-body :global(.hljs-built_in)     { color: #00d4ff; }   /* cyan */

  /* String literals */
  .code-body :global(.hljs-string),
  .code-body :global(.hljs-template-string),
  .code-body :global(.hljs-template-tag) { color: #00ff88; }   /* neon green */

  /* Numeric & boolean literals */
  .code-body :global(.hljs-number),
  .code-body :global(.hljs-literal)      { color: #ffaa44; }   /* amber */

  /* Comments */
  .code-body :global(.hljs-comment),
  .code-body :global(.hljs-quote)        { color: #4a5580; font-style: italic; }

  /* Function / method names at call/definition sites */
  .code-body :global(.hljs-title),
  .code-body :global(.hljs-title\.function_),
  .code-body :global(.hljs-title\.class_) { color: #88ccff; }  /* light blue */

  /* Type names, class names */
  .code-body :global(.hljs-type),
  .code-body :global(.hljs-class .hljs-title) { color: #cc88ff; }  /* lavender */

  /* Variables, params */
  .code-body :global(.hljs-variable),
  .code-body :global(.hljs-params)       { color: #e0e0f0; }

  /* Attributes / decorators */
  .code-body :global(.hljs-attr),
  .code-body :global(.hljs-attribute)    { color: #ffdd66; }   /* warm yellow */

  /* Operators, punctuation */
  .code-body :global(.hljs-operator),
  .code-body :global(.hljs-punctuation)  { color: #88a0b8; }

  /* Preprocessor / meta */
  .code-body :global(.hljs-meta),
  .code-body :global(.hljs-meta .hljs-keyword) { color: #ff8844; }  /* orange */

  /* Tags in XML/HTML/Svelte */
  .code-body :global(.hljs-tag)          { color: #00d4ff; }
  .code-body :global(.hljs-tag .hljs-name) { color: #88ccff; }
  .code-body :global(.hljs-tag .hljs-attr) { color: #ffdd66; }

  /* Diff colors */
  .code-body :global(.hljs-addition)     { color: #00ff88; }
  .code-body :global(.hljs-deletion)     { color: #ff2244; }

  /* Section headers (ini, markdown) */
  .code-body :global(.hljs-section)      { color: #00d4ff; font-weight: bold; }

  /* Markdown bold/emphasis */
  .code-body :global(.hljs-strong)       { font-weight: bold; }
  .code-body :global(.hljs-emphasis)     { font-style: italic; }

  /* ── Rendered markdown view ─────────────────────────────────────────────── */
  .md-body {
    flex: 1;
    overflow-y: auto;
    padding: 28px 40px;
    scrollbar-width: thin;
    scrollbar-color: var(--scrollbar-thumb) var(--bg);
    font-family: -apple-system, 'Segoe UI', system-ui, sans-serif;
    font-size: 14px;
    line-height: 1.7;
    color: var(--text);
  }

  .md-body :global(h1) {
    font-family: 'Courier New', monospace;
    font-size: 22px;
    color: var(--accent);
    margin: 0 0 20px;
    padding-bottom: 10px;
    border-bottom: 1px solid var(--border-dim);
  }
  .md-body :global(h2) {
    font-family: 'Courier New', monospace;
    font-size: 15px;
    color: var(--accent);
    margin: 32px 0 10px;
    padding-bottom: 5px;
    border-bottom: 1px solid var(--border-dim);
    text-transform: uppercase;
    letter-spacing: 0.07em;
  }
  .md-body :global(h3) {
    font-family: 'Courier New', monospace;
    font-size: 13px;
    color: var(--text);
    font-weight: bold;
    margin: 22px 0 8px;
  }
  .md-body :global(h4),
  .md-body :global(h5),
  .md-body :global(h6) {
    font-family: 'Courier New', monospace;
    color: var(--text-dim);
    margin: 16px 0 6px;
  }
  .md-body :global(p)  { margin: 0 0 14px; }
  .md-body :global(ul),
  .md-body :global(ol) { margin: 0 0 14px; padding-left: 22px; }
  .md-body :global(li) { margin: 5px 0; }
  .md-body :global(li > p) { margin: 0; }

  .md-body :global(code) {
    font-family: 'Courier New', monospace;
    font-size: 12.5px;
    background: rgba(0,212,255,0.08);
    color: var(--accent);
    padding: 1px 6px;
    border-radius: 2px;
    border: 1px solid rgba(0,212,255,0.15);
  }
  .md-body :global(pre) {
    background: var(--bg-panel);
    border: 1px solid var(--border-dim);
    padding: 14px 18px;
    overflow-x: auto;
    margin: 12px 0 18px;
    scrollbar-width: thin;
    scrollbar-color: var(--scrollbar-thumb) var(--bg);
  }
  .md-body :global(pre code) {
    background: none;
    border: none;
    padding: 0;
    color: #a8b8c8;
    font-size: 12px;
    line-height: 1.55;
  }

  .md-body :global(table) {
    width: 100%;
    border-collapse: collapse;
    margin: 12px 0 18px;
    font-size: 13px;
  }
  .md-body :global(th) {
    background: var(--bg-header);
    color: var(--accent);
    font-family: 'Courier New', monospace;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.07em;
    padding: 6px 12px;
    border: 1px solid var(--border-dim);
    text-align: left;
  }
  .md-body :global(td) {
    padding: 6px 12px;
    border: 1px solid var(--border-dim);
    vertical-align: top;
  }
  .md-body :global(tr:nth-child(even) td) {
    background: rgba(255,255,255,0.025);
  }

  .md-body :global(blockquote) {
    border-left: 3px solid var(--accent);
    margin: 14px 0;
    padding: 8px 18px;
    background: rgba(0,212,255,0.04);
    color: var(--text-dim);
    font-style: italic;
  }
  .md-body :global(blockquote p) { margin: 0; color: inherit; }

  .md-body :global(hr) {
    border: none;
    border-top: 1px solid var(--border-dim);
    margin: 28px 0;
  }
  .md-body :global(a) {
    color: var(--accent);
    text-decoration: none;
  }
  .md-body :global(a:hover) { text-decoration: underline; }

  .md-body :global(strong) { font-weight: bold; color: var(--text); }
  .md-body :global(em)     { font-style: italic; color: var(--text-dim); }

  .md-body :global(img) {
    max-width: 100%;
    height: auto;
    display: block;
    margin: 12px 0;
    border: 1px solid var(--border-dim);
  }

  .md-body :global(input[type="checkbox"]) {
    margin-right: 6px;
  }
</style>
