<script>
  import { S, undo, redo, isDirty, loadScene, saveScene } from '../lib/state.svelte.js';
  import { testTone, refreshOutputs } from '../lib/engine.js';
  import Icon from './Icon.svelte';

  let dirty = $derived(isDirty());
  let devOpen = $state(false);
  let dragIdx = $state(-1);
  let latency = $derived(((S.settings.buf / S.settings.sr) * 1000).toFixed(1));

  // Device list: real outputs in the Windows app, examples in the web demo.
  let outputs = $derived(S.rt.live ? S.rt.outputs.map((o) => o.name)
    : ['Headphones (USB Audio)', 'Speakers (Realtek Audio)', 'Monitor (HDMI Audio)']);
  let current = $derived(S.rt.live ? (S.rt.outputs.find((o) => o.is_default)?.name ?? 'No output found') : outputs[0]);
  // Fallback order: saved order first, then any new devices.
  let fallback = $derived.by(() => {
    const others = outputs.filter((n) => n !== current);
    const saved = S.devices.fallback.filter((n) => others.includes(n));
    return [...saved, ...others.filter((n) => !saved.includes(n))];
  });
  function moveFallback(from, to) {
    const list = [...fallback];
    const [x] = list.splice(from, 1);
    list.splice(to, 0, x);
    S.devices.fallback = list;
  }
  let naming = $state(false);
  let newName = $state('');
  function finishSaveAs(ok) {
    if (ok && newName.trim()) saveScene(newName.trim());
    naming = false; newName = '';
  }
  const focus = (n) => n.focus();
  let health = $derived(S.rt.live
    ? (S.rt.error ? { c: 'var(--hot)', t: 'Engine error' } : { c: 'var(--ok)', t: `Live · ${S.rt.sessions.length} apps · ${S.rt.pollMs} ms` })
    : { c: 'var(--warn)', t: 'Demo signal' });
</script>

<header class="top">
  <div class="brand"><span class="logo" aria-hidden="true"><i></i><i></i><i></i></span>Daelij MiX</div>

  <div class="scene">
    <label class="sr-only" for="scene">Scene</label>
    <select id="scene" class="select" value={S.scenes.current} onchange={(e) => loadScene(e.currentTarget.value)}>
      {#each S.scenes.list as sc}<option value={sc.name}>{sc.name}</option>{/each}
    </select>
    {#if dirty}<span class="dirty" title="This scene has unsaved changes" aria-label="Unsaved changes"></span>{/if}
    <button class="iconbtn" onclick={() => saveScene()} disabled={!dirty} title="Save scene" aria-label="Save scene"><Icon name="save" /></button>
    {#if naming}
      <input class="select nm" bind:value={newName} use:focus placeholder="Scene name" aria-label="New scene name"
        onkeydown={(e) => { e.stopPropagation(); if (e.key === 'Enter') finishSaveAs(true); if (e.key === 'Escape') finishSaveAs(false); }}
        onblur={() => finishSaveAs(true)} />
    {:else}
      <button class="btn ghost small" onclick={() => (naming = true)}>Save as…</button>
    {/if}
  </div>

  <div class="ur">
    <button class="iconbtn" onclick={undo} disabled={!S.rt.canUndo} title="Undo (Ctrl+Z)" aria-label="Undo"><Icon name="undo" /></button>
    <button class="iconbtn" onclick={redo} disabled={!S.rt.canRedo} title="Redo (Ctrl+Y)" aria-label="Redo"><Icon name="redo" /></button>
  </div>

  <div class="health" title={S.rt.error || health.t}><b style="background:{health.c}"></b>{health.t}</div>

  <div class="spacer"></div>

  <div class="device">
    <button class="devbtn" onclick={() => { devOpen = !devOpen; if (devOpen) refreshOutputs(); }} aria-expanded={devOpen} aria-haspopup="dialog">
      <Icon name="speaker" /><span class="dname">{current}</span><span class="lat">{latency} ms</span><Icon name="chevron" size={14} />
    </button>
    <button class="btn small" onclick={testTone} title="Play a short test tone"><Icon name="play" size={12} />Test</button>
    {#if devOpen}
      <div class="pop" role="dialog" aria-label="Output devices">
        <div class="cur"><b style="background:{S.rt.live ? 'var(--ok)' : 'var(--warn)'}"></b>
          <div><small>Current output</small><span>{current}</span></div></div>
        <p class="note">{S.rt.live ? 'Change the default device in Windows sound settings. Fallback order is saved for when devices change.' : 'Example devices in the web demo.'}</p>
        <h5>Fallback order <small>drag to reorder</small></h5>
        <ol>
          {#each fallback as d, i (d)}
            <li draggable="true" class:drag={dragIdx === i}
              ondragstart={() => (dragIdx = i)} ondragend={() => (dragIdx = -1)}
              ondragover={(e) => { e.preventDefault(); if (dragIdx >= 0 && dragIdx !== i) { moveFallback(dragIdx, i); dragIdx = i; } }}>
              <Icon name="grip" size={14} /><span>{i + 1}. {d}</span>
              <span class="arrows">
                <button disabled={i === 0} onclick={() => moveFallback(i, i - 1)} aria-label="Move {d} up">↑</button>
                <button disabled={i === fallback.length - 1} onclick={() => moveFallback(i, i + 1)} aria-label="Move {d} down">↓</button>
              </span>
            </li>
          {:else}<li class="none">No other devices</li>{/each}
        </ol>
      </div>
    {/if}
  </div>
</header>

<svelte:window onclick={(e) => { if (devOpen && !e.target.closest('.device')) devOpen = false; }} />

<style>
  .top { display: flex; align-items: center; gap: 14px; padding: 0 14px; height: 50px; background: var(--bar); border-bottom: 1px solid var(--line); }
  .brand { display: flex; align-items: center; gap: 9px; font-weight: 700; font-size: 14px; white-space: nowrap; }
  .logo { display: flex; gap: 2px; align-items: flex-end; height: 15px; }
  .logo i { width: 3px; border-radius: 2px; display: block; }
  .logo i:nth-child(1) { height: 8px; background: var(--accent); }
  .logo i:nth-child(2) { height: 15px; background: #5ee08a; }
  .logo i:nth-child(3) { height: 11px; background: #ff5577; }
  .scene { display: flex; align-items: center; gap: 6px; }
  .scene select { width: 130px; }
  .nm { width: 120px; }
  .dirty { width: 8px; height: 8px; border-radius: 50%; background: var(--warn); box-shadow: 0 0 0 3px rgba(255,209,102,.18); }
  .small { height: 28px; padding: 0 9px; font-size: 11.5px; }
  .ur { display: flex; gap: 2px; }
  .health { display: flex; align-items: center; gap: 7px; font-size: 11.5px; color: var(--muted); border: 1px solid var(--line-2); border-radius: 99px; padding: 3px 10px; white-space: nowrap; }
  .health b { width: 7px; height: 7px; border-radius: 50%; display: block; }
  .spacer { flex: 1; }
  .device { position: relative; display: flex; gap: 6px; align-items: center; }
  .devbtn { display: flex; align-items: center; gap: 8px; height: 30px; padding: 0 10px; border-radius: 8px; border: 1px solid var(--line-2); background: var(--well); max-width: 300px; }
  .dname { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; font-size: 12px; }
  .lat { color: var(--muted); font-size: 11px; font-variant-numeric: tabular-nums; white-space: nowrap; }
  .pop { position: absolute; right: 0; top: 38px; width: 320px; z-index: 20; background: var(--surface); border: 1px solid var(--line-2); border-radius: 12px; padding: 14px; box-shadow: 0 20px 40px rgba(0,0,0,.5); }
  .cur { display: flex; align-items: center; gap: 10px; }
  .cur b { width: 8px; height: 8px; border-radius: 50%; display: block; }
  .cur small { display: block; color: var(--muted); font-size: 10.5px; }
  .note { color: var(--dim); font-size: 11px; margin: 8px 0 12px; }
  h5 { margin: 0 0 6px; font-size: 12px; }
  h5 small { color: var(--dim); font-weight: 500; }
  ol { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 4px; }
  li { display: flex; align-items: center; gap: 8px; padding: 6px 8px; border-radius: 7px; background: var(--strip); border: 1px solid var(--line); font-size: 12px; cursor: grab; }
  li.drag { opacity: .5; }
  li span:nth-child(2) { flex: 1; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  li.none { color: var(--dim); cursor: default; }
  .arrows { display: flex; gap: 2px; }
  .arrows button { width: 20px; height: 20px; border-radius: 5px; color: var(--muted); }
  .arrows button:hover:not(:disabled) { background: rgba(255,255,255,.08); }
  @media (max-width: 1100px) { .health { display: none; } }
  @media (max-width: 860px) { .scene .btn, .lat { display: none; } .dname { max-width: 110px; } }
</style>
