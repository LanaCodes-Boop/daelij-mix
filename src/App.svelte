<script>
  import { S, undo, redo, save, label } from './lib/state.svelte.js';
  import { resetPeaks, isTauri } from './lib/engine.js';
  import TopBar from './components/TopBar.svelte';
  import SourceStrip from './components/SourceStrip.svelte';
  import MixStrip from './components/MixStrip.svelte';
  import Inspector from './components/Inspector.svelte';
  import MatrixView from './components/MatrixView.svelte';
  import AppsView from './components/AppsView.svelte';
  import SettingsView from './components/SettingsView.svelte';
  import Icon from './components/Icon.svelte';

  const VIEWS = [['mixer', 'Mixer'], ['matrix', 'Matrix'], ['apps', 'Applications'], ['settings', 'Settings']];
  let focusMix = $derived(S.mix.mixes.find((m) => m.id === S.ui.focus));
  let showInspector = $derived(S.ui.view === 'mixer' && S.ui.density !== 'compact' && S.ui.inspector);
  let latency = $derived(((S.settings.buf / S.settings.sr) * 1000).toFixed(1));

  function onKey(e) {
    const typing = e.target.closest?.('input, select, textarea');
    const mod = e.ctrlKey || e.metaKey;
    if (mod && !typing && e.key.toLowerCase() === 'z' && !e.shiftKey) { e.preventDefault(); undo(); }
    else if (mod && !typing && (e.key.toLowerCase() === 'y' || (e.key.toLowerCase() === 'z' && e.shiftKey))) { e.preventDefault(); redo(); }
    else if (!mod && !typing && (e.key === 'r' || e.key === 'R')) resetPeaks();
  }
  $effect(() => { S.ui.density; S.ui.inspector; save(); });
</script>

<svelte:window onkeydown={onKey} />

<div class="shell">
  <TopBar />

  <nav class="tabs">
    <div role="tablist" aria-label="Views">
      {#each VIEWS as [id, name]}
        <button role="tab" aria-selected={S.ui.view === id} onclick={() => (S.ui.view = id)}>{name}</button>
      {/each}
    </div>
    <div class="spacer"></div>
    {#if S.ui.view === 'mixer'}
      <div class="seg" role="group" aria-label="Density">
        {#each [['compact', 'Compact'], ['standard', 'Standard'], ['detailed', 'Detailed']] as [id, name]}
          <button aria-pressed={S.ui.density === id} onclick={() => (S.ui.density = id)}>{name}</button>
        {/each}
      </div>
      <button class="iconbtn" aria-pressed={S.ui.inspector} disabled={S.ui.density === 'compact'}
        onclick={() => (S.ui.inspector = !S.ui.inspector)} title="Show or hide the inspector" aria-label="Inspector"><Icon name="inspector" /></button>
    {/if}
  </nav>

  <main class="view" class:pad={S.ui.view !== 'mixer'}>
    {#if S.ui.view === 'mixer'}
      <div class="mixer">
        <div class="strips-scroll">
          <div class="groups">
            <section class="group" aria-label="Source buses">
              <div class="glabel">SOURCES <span style="color:{focusMix.color}">· faders show the sends to {label(focusMix)}</span></div>
              <div class="strips">{#each S.mix.sources as src (src.id)}<SourceStrip {src} />{/each}</div>
            </section>
            <section class="group outs" aria-label="Mixes">
              <div class="glabel">MIXES <span>· pick one to focus</span></div>
              <div class="strips">{#each S.mix.mixes as mix (mix.id)}<MixStrip {mix} />{/each}</div>
            </section>
          </div>
        </div>
        {#if showInspector}<Inspector />{/if}
      </div>
    {:else if S.ui.view === 'matrix'}<MatrixView />
    {:else if S.ui.view === 'apps'}<AppsView />
    {:else}<SettingsView />{/if}
  </main>

  <footer class="status">
    <span>{S.settings.sr / 1000} kHz</span><span>{S.settings.buf} smp</span><span>{latency} ms</span>
    <span>Remote clients: 0</span>
    <span class="mode">{isTauri ? (S.rt.live ? 'Windows audio' : 'Starting…') : 'Web demo'}</span>
    <span class="keys">Shift-drag = fine · double-click a value to type · Ctrl+Z / Ctrl+Y · R resets peaks</span>
  </footer>
</div>

<style>
  .shell { height: 100%; display: grid; grid-template-rows: auto auto 1fr auto; }
  .tabs { display: flex; align-items: center; gap: 10px; padding: 8px 14px; border-bottom: 1px solid var(--line); background: var(--surface); }
  [role=tablist] { display: flex; gap: 4px; }
  [role=tab] { padding: 6px 13px; border-radius: 8px; color: var(--muted); font-weight: 600; }
  [role=tab]:hover { color: var(--text); background: rgba(255,255,255,.04); }
  [role=tab][aria-selected=true] { color: var(--text); background: rgba(139,156,255,.14); box-shadow: inset 0 0 0 1px rgba(139,156,255,.35); }
  .spacer { flex: 1; }
  .iconbtn[aria-pressed=true] { color: var(--text); background: rgba(255,255,255,.07); }
  .view { min-height: 0; overflow: auto; }
  .view.pad { padding: 16px; }
  .mixer { display: flex; gap: 12px; height: 100%; padding: 12px 14px; min-height: 0; }
  .strips-scroll { flex: 1; min-width: 0; overflow-x: auto; overflow-y: hidden; }
  .groups { display: flex; gap: 14px; height: 100%; min-height: 540px; }
  .group { display: flex; flex-direction: column; gap: 8px; flex: 4 1 0; min-width: 0; }
  .group.outs { flex: 2 1 0; }
  .glabel { font-size: 10.5px; font-weight: 700; letter-spacing: 1.3px; color: var(--dim); padding-left: 4px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .glabel span { font-weight: 600; letter-spacing: .2px; color: var(--muted); }
  .strips { display: flex; gap: 8px; flex: 1; min-height: 0; }
  .mixer :global(.insp) { height: 100%; }
  .status { display: flex; align-items: center; gap: 16px; padding: 0 14px; height: 34px; border-top: 1px solid var(--line); background: var(--surface); font-size: 11.5px; color: var(--muted); font-variant-numeric: tabular-nums; white-space: nowrap; overflow: hidden; }
  .mode { color: var(--text); }
  .keys { margin-left: auto; color: var(--dim); overflow: hidden; text-overflow: ellipsis; }
  @media (max-width: 900px) {
    .tabs { flex-wrap: wrap; }
    .mixer :global(.insp) { position: fixed; inset: 0; width: auto; z-index: 30; border-radius: 0; height: auto; }
    .keys { display: none; }
  }
</style>
