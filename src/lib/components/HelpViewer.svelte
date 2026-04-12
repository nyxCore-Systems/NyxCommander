<script lang="ts">
  import { createEventDispatcher, onMount, tick } from 'svelte';
  import { marked } from 'marked';
  import { invoke } from '@tauri-apps/api/core';

  // Import all doc pages as raw strings via Vite's ?raw loader
  import overviewMd    from '../../../docs/user/01-overview.md?raw';
  import navigationMd  from '../../../docs/user/02-navigation.md?raw';
  import fileOpsMd     from '../../../docs/user/03-file-ops.md?raw';
  import archivesMd    from '../../../docs/user/04-archives.md?raw';
  import viewersMd     from '../../../docs/user/05-viewers.md?raw';
  import toolsMd       from '../../../docs/user/06-tools.md?raw';
  import customizeMd   from '../../../docs/user/07-customization.md?raw';
  import shortcutsMd   from '../../../docs/user/08-shortcuts.md?raw';
  import pluginsMd     from '../../../docs/user/plugins/index.md?raw';
  import pluginsDevMd  from '../../../docs/user/plugins/authoring.md?raw';

  const dispatch = createEventDispatcher<{ close: void }>();

  interface Section { id: string; title: string; raw: string; }

  const sections: Section[] = [
    { id: 'overview',      title: '1. Overview',         raw: overviewMd },
    { id: 'navigation',    title: '2. Navigation',        raw: navigationMd },
    { id: 'file-ops',      title: '3. File Operations',   raw: fileOpsMd },
    { id: 'archives',      title: '4. Archives',          raw: archivesMd },
    { id: 'viewers',       title: '5. Viewers',           raw: viewersMd },
    { id: 'tools',         title: '6. Tools',             raw: toolsMd },
    { id: 'customization', title: '7. Customization',     raw: customizeMd },
    { id: 'shortcuts',     title: '8. Shortcuts',         raw: shortcutsMd },
    { id: 'plugins',       title: '9. Plugins',           raw: pluginsMd },
    { id: 'plugin-dev',    title: '10. Plugin Dev',       raw: pluginsDevMd },
  ];

  marked.setOptions({ breaks: true, gfm: true });

  // ── State ────────────────────────────────────────────────────────────────────

  let activeIdx = 0;
  let focus: 'sidebar' | 'content' = 'sidebar';

  // Search
  let searchQuery = '';
  let searchInputEl: HTMLInputElement;

  // Link behaviour: true = Enter opens in Nyx (section nav / file viewer),
  //                 Cmd+Enter opens in system.
  //                 false = inverted.
  let linksDefaultNyx = true;

  let overlayEl: HTMLElement;
  let contentEl: HTMLElement;

  // ── Search ───────────────────────────────────────────────────────────────────

  // Count case-insensitive occurrences of query in a string.
  function countMatches(text: string, q: string): number {
    if (!q) return 0;
    let count = 0;
    const lower = text.toLowerCase();
    const ql = q.toLowerCase();
    let pos = 0;
    while ((pos = lower.indexOf(ql, pos)) !== -1) { count++; pos += ql.length; }
    return count;
  }

  // Wrap query occurrences in <mark> tags (case-insensitive, HTML-safe).
  function highlight(html: string, q: string): string {
    if (!q || q.length < 2) return html;
    // Work on text nodes only — avoid breaking tags.
    // Simple approach: split on tag boundaries and only process text chunks.
    return html.replace(/(<[^>]+>)|([^<]+)/g, (_, tag, text) => {
      if (tag) return tag;
      const escaped = q.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
      return text.replace(new RegExp(escaped, 'gi'), (m: string) => `<mark>${m}</mark>`);
    });
  }

  $: matchCounts = sections.map(s => countMatches(s.raw, searchQuery));

  // Sections visible in sidebar: all when no query; only those with hits when searching.
  $: visibleSections = searchQuery.length >= 2
    ? sections.map((s, i) => ({ ...s, origIdx: i })).filter((_, i) => matchCounts[i] > 0)
    : sections.map((s, i) => ({ ...s, origIdx: i }));

  // ── Rendered HTML ────────────────────────────────────────────────────────────

  $: baseHtml = marked.parse(sections[activeIdx].raw) as string;
  $: html = highlight(baseHtml, searchQuery);

  // Scroll content to top when section changes; scroll first match into view when searching.
  $: {
    if (contentEl && html) {
      tick().then(() => {
        if (searchQuery.length >= 2) {
          const first = contentEl.querySelector('mark');
          first?.scrollIntoView({ block: 'center', behavior: 'smooth' });
        } else {
          contentEl.scrollTop = 0;
        }
      });
    }
  }

  // ── Focus / navigation helpers ───────────────────────────────────────────────

  function goToSection(idx: number) {
    activeIdx = Math.max(0, Math.min(sections.length - 1, idx));
  }

  function scrollContent(delta: number) {
    contentEl?.scrollBy({ top: delta, behavior: 'instant' });
  }

  // ── Link interception ────────────────────────────────────────────────────────

  // Slugify a heading text the same way marked does (lowercase, strip non-alphanumeric).
  function slug(text: string): string {
    return text.toLowerCase().replace(/[^\w\s-]/g, '').replace(/\s+/g, '-');
  }

  function scrollToFragment(id: string) {
    const el = contentEl?.querySelector(`#${id}, [id="${id}"]`);
    if (el) {
      el.scrollIntoView({ block: 'start', behavior: 'smooth' });
    } else {
      // Fallback: find heading whose text slugifies to id.
      const headings = contentEl?.querySelectorAll('h1,h2,h3,h4') ?? [];
      for (const h of headings) {
        if (slug(h.textContent ?? '') === id) {
          h.scrollIntoView({ block: 'start', behavior: 'smooth' });
          return;
        }
      }
    }
  }

  function handleLinkClick(e: MouseEvent) {
    const anchor = (e.target as Element).closest('a');
    if (!anchor) return;

    const href = anchor.getAttribute('href') ?? '';
    if (!href) return;
    e.preventDefault();

    // Modifier logic: Cmd/Ctrl inverts the default.
    const cmdHeld = e.metaKey || e.ctrlKey;
    const openInNyx = cmdHeld ? !linksDefaultNyx : linksDefaultNyx;

    // Pure fragment: #heading-id
    if (href.startsWith('#')) {
      scrollToFragment(href.slice(1));
      return;
    }

    // Relative .md link: navigate to matching section (optionally with fragment).
    const mdMatch = href.match(/^([^#]+\.mdx?)(?:#(.+))?$/i);
    if (mdMatch) {
      const file = mdMatch[1].replace(/^.*\//, ''); // basename
      const fragment = mdMatch[2] ?? '';
      const targetIdx = sections.findIndex(s =>
        s.id === file.replace(/^\d+-/, '').replace(/\.mdx?$/, '')
        || file.includes(s.id)
      );
      if (targetIdx !== -1) {
        goToSection(targetIdx);
        if (fragment) tick().then(() => scrollToFragment(fragment));
      }
      return;
    }

    // External URL or anything else.
    if (openInNyx) {
      // "Open in Nyx" for http(s) links means open with the system browser via Tauri opener —
      // there is no embedded web renderer. For file:// or absolute paths a viewer could be used,
      // but docs don't contain those. Treat as system open for now.
      invoke('open_file', { path: href }).catch(console.error);
    } else {
      invoke('open_file', { path: href }).catch(console.error);
    }
  }

  // ── Keyboard ─────────────────────────────────────────────────────────────────

  onMount(() => overlayEl?.focus());

  function handleKey(e: KeyboardEvent) {
    e.stopPropagation();

    // Always: Esc closes (or clears search first if active).
    if (e.key === 'Escape') {
      if (searchQuery) { searchQuery = ''; return; }
      dispatch('close');
      return;
    }

    // '/' focuses search from anywhere (unless already in search input).
    if (e.key === '/' && document.activeElement !== searchInputEl) {
      e.preventDefault();
      searchInputEl?.focus();
      return;
    }

    // While search is focused, don't intercept printable keys.
    if (document.activeElement === searchInputEl) return;

    // Left → sidebar focus, Right → content focus.
    if (e.key === 'ArrowLeft') {
      e.preventDefault();
      focus = 'sidebar';
      return;
    }
    if (e.key === 'ArrowRight') {
      e.preventDefault();
      focus = 'content';
      return;
    }

    if (focus === 'sidebar') {
      if (e.key === 'ArrowUp') {
        e.preventDefault();
        // Navigate up through visible sections.
        const ci = visibleSections.findIndex(s => s.origIdx === activeIdx);
        if (ci > 0) goToSection(visibleSections[ci - 1].origIdx);
        return;
      }
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        const ci = visibleSections.findIndex(s => s.origIdx === activeIdx);
        const next = visibleSections[ci + 1];
        if (next) goToSection(next.origIdx);
        return;
      }
      if (e.key === 'Enter') {
        e.preventDefault();
        focus = 'content';
        return;
      }
      // Number keys jump directly.
      const n = parseInt(e.key);
      if (!isNaN(n) && n >= 1 && n <= sections.length) {
        goToSection(n - 1);
      }
    } else {
      // Content focus — Up/Down scroll the article.
      const SCROLL_LINE = 60;
      const SCROLL_PAGE = contentEl ? contentEl.clientHeight * 0.85 : 400;
      if (e.key === 'ArrowUp') { e.preventDefault(); scrollContent(-SCROLL_LINE); return; }
      if (e.key === 'ArrowDown') { e.preventDefault(); scrollContent(SCROLL_LINE); return; }
      if (e.key === 'PageUp') { e.preventDefault(); scrollContent(-SCROLL_PAGE); return; }
      if (e.key === 'PageDown') { e.preventDefault(); scrollContent(SCROLL_PAGE); return; }
    }
  }
</script>

<!-- svelte-ignore a11y-no-noninteractive-element-interactions a11y-no-noninteractive-tabindex -->
<div class="overlay" role="dialog" tabindex="-1" bind:this={overlayEl} on:keydown={handleKey}>
  <div class="dialog">

    <div class="titlebar">
      <span class="title">Nyx.Commander</span>

      <!-- Search field -->
      <div class="search-wrap" class:has-query={searchQuery.length > 0}>
        <span class="search-icon">⌕</span>
        <input
          bind:this={searchInputEl}
          bind:value={searchQuery}
          class="search-input"
          placeholder="Search docs…  /"
          spellcheck="false"
          on:keydown={e => { if (e.key === 'Escape') { e.stopPropagation(); searchQuery = ''; overlayEl?.focus(); } }}
        />
        {#if searchQuery}
          <span class="search-count">
            {matchCounts.reduce((a, b) => a + b, 0)} hits
          </span>
          <button class="search-clear" on:click={() => { searchQuery = ''; overlayEl?.focus(); }}>✕</button>
        {/if}
      </div>

      <!-- Link-open toggle -->
      <button
        class="link-toggle"
        class:active={linksDefaultNyx}
        title="Links: {linksDefaultNyx ? 'Enter=Nyx · Cmd+Enter=System' : 'Enter=System · Cmd+Enter=Nyx'} — click to toggle"
        on:click={() => linksDefaultNyx = !linksDefaultNyx}
      >
        {linksDefaultNyx ? '⇥ Nyx' : '⇥ Sys'}
      </button>

      <span class="hint">
        {focus === 'sidebar' ? '◀ sidebar' : '▶ content'} · ←→ switch · / search · Esc close
      </span>
      <button class="close-btn" on:click={() => dispatch('close')}>✕</button>
    </div>

    <div class="body">
      <!-- Sidebar -->
      <nav class="sidebar" class:focused={focus === 'sidebar'}>
        {#each visibleSections as sec (sec.origIdx)}
          <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
          <div
            class="sidebar-item"
            class:active={sec.origIdx === activeIdx}
            on:click={() => { goToSection(sec.origIdx); focus = 'content'; }}
          >
            <span class="sec-title">{sec.title}</span>
            {#if searchQuery.length >= 2 && matchCounts[sec.origIdx] > 0}
              <span class="match-badge">{matchCounts[sec.origIdx]}</span>
            {/if}
          </div>
        {/each}
        {#if searchQuery.length >= 2 && visibleSections.length === 0}
          <div class="no-results">No results</div>
        {/if}
      </nav>

      <!-- Content -->
      <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
      <article
        class="content"
        class:focused={focus === 'content'}
        bind:this={contentEl}
        on:click={handleLinkClick}
      >
        <!-- eslint-disable-next-line svelte/no-at-html-tags -->
        {@html html}
      </article>
    </div>

  </div>
</div>

<style>
  /* ── Shell ──────────────────────────────────────────────────────────────── */
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0,0,0,0.82);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 200;
    outline: none;
  }

  .dialog {
    background: var(--bg-panel);
    border: 1px solid var(--border-panel);
    width: min(900px, 96vw);
    height: min(680px, 90vh);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  /* ── Titlebar ────────────────────────────────────────────────────────────── */
  .titlebar {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 7px 12px;
    background: var(--bg-header);
    border-bottom: 1px solid var(--border-panel);
    flex-shrink: 0;
  }
  .title {
    color: var(--accent);
    font-family: 'Courier New', monospace;
    font-size: 12px;
    font-weight: bold;
    letter-spacing: 0.05em;
  }
  .hint {
    flex: 1;
    color: var(--text-dim);
    font-family: 'Courier New', monospace;
    font-size: 10px;
    text-align: right;
  }
  .close-btn {
    background: none;
    border: none;
    color: var(--text-dim);
    cursor: pointer;
    font-size: 16px;
    padding: 0;
    flex-shrink: 0;
  }
  .close-btn:hover { color: var(--danger, #ff2244); }

  /* ── Layout ──────────────────────────────────────────────────────────────── */
  .body {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  /* ── Sidebar ─────────────────────────────────────────────────────────────── */
  .sidebar {
    width: 180px;
    flex-shrink: 0;
    border-right: 1px solid var(--border-dim);
    overflow-y: auto;
    padding: 8px 0;
    background: var(--bg);
    scrollbar-width: thin;
    scrollbar-color: var(--scrollbar-thumb) var(--bg);
  }

  .sidebar-item {
    padding: 7px 14px;
    font-family: 'Courier New', monospace;
    font-size: 11px;
    color: var(--text-dim);
    cursor: pointer;
    border-left: 2px solid transparent;
    user-select: none;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .sidebar-item:hover { color: var(--text); background: var(--bg-panel); }
  .sidebar-item.active {
    color: var(--accent);
    border-left-color: var(--accent);
    background: var(--bg-panel);
  }

  /* ── Content ─────────────────────────────────────────────────────────────── */
  .content {
    flex: 1;
    overflow-y: auto;
    padding: 24px 32px;
    scrollbar-width: thin;
    scrollbar-color: var(--scrollbar-thumb) var(--bg-panel);
    font-family: -apple-system, 'Segoe UI', system-ui, sans-serif;
    font-size: 13px;
    line-height: 1.65;
    color: var(--text);
  }

  /* ── Markdown styles (scoped to .content) ─────────────────────────────────── */
  .content :global(h1) {
    font-family: 'Courier New', monospace;
    font-size: 20px;
    color: var(--accent);
    margin: 0 0 20px 0;
    padding-bottom: 8px;
    border-bottom: 1px solid var(--border-dim);
    letter-spacing: 0.03em;
  }

  .content :global(h2) {
    font-family: 'Courier New', monospace;
    font-size: 14px;
    color: var(--accent);
    margin: 28px 0 10px 0;
    padding-bottom: 4px;
    border-bottom: 1px solid var(--border-dim);
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }

  .content :global(h3) {
    font-family: 'Courier New', monospace;
    font-size: 12px;
    color: var(--text);
    margin: 20px 0 8px 0;
    font-weight: bold;
  }

  .content :global(p) {
    margin: 0 0 12px 0;
    color: var(--text);
  }

  .content :global(ul),
  .content :global(ol) {
    margin: 0 0 12px 0;
    padding-left: 20px;
  }

  .content :global(li) {
    margin: 4px 0;
    color: var(--text);
  }

  .content :global(code) {
    font-family: 'Courier New', monospace;
    font-size: 12px;
    background: rgba(0,212,255,0.08);
    color: var(--accent);
    padding: 1px 5px;
    border-radius: 2px;
    border: 1px solid rgba(0,212,255,0.15);
  }

  .content :global(pre) {
    background: var(--bg);
    border: 1px solid var(--border-dim);
    padding: 14px 16px;
    overflow-x: auto;
    margin: 12px 0 16px 0;
    scrollbar-width: thin;
    scrollbar-color: var(--scrollbar-thumb) var(--bg);
  }

  .content :global(pre code) {
    background: none;
    border: none;
    padding: 0;
    color: var(--text-dim);
    font-size: 11px;
    line-height: 1.5;
  }

  .content :global(table) {
    width: 100%;
    border-collapse: collapse;
    margin: 12px 0 16px 0;
    font-size: 12px;
  }

  .content :global(th) {
    background: var(--bg-header);
    color: var(--accent);
    font-family: 'Courier New', monospace;
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    padding: 6px 10px;
    border: 1px solid var(--border-dim);
    text-align: left;
  }

  .content :global(td) {
    padding: 5px 10px;
    border: 1px solid var(--border-dim);
    color: var(--text);
    vertical-align: top;
  }

  .content :global(tr:nth-child(even) td) {
    background: rgba(255,255,255,0.02);
  }

  .content :global(td code) {
    font-size: 11px;
  }

  .content :global(blockquote) {
    border-left: 3px solid var(--accent);
    margin: 12px 0;
    padding: 8px 16px;
    background: rgba(0,212,255,0.04);
    color: var(--text-dim);
    font-style: italic;
  }

  .content :global(blockquote p) {
    margin: 0;
    color: inherit;
  }

  .content :global(blockquote strong) {
    color: var(--accent);
    font-style: normal;
  }

  .content :global(hr) {
    border: none;
    border-top: 1px solid var(--border-dim);
    margin: 24px 0;
  }

  .content :global(a) {
    color: var(--accent);
    text-decoration: none;
  }
  .content :global(a:hover) {
    text-decoration: underline;
  }

  .content :global(strong) {
    color: var(--text);
    font-weight: bold;
  }

  .content :global(em) {
    font-style: italic;
    color: var(--text-dim);
  }
</style>
