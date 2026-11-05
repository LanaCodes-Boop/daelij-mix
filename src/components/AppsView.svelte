<script>
  import { S, label, assignedSource, assignApp, appVol, appMuted, hasRule, save } from '../lib/state.svelte.js';
  import { currentApps, resetAppVolumes } from '../lib/engine.js';
  import Meter from './Meter.svelte';
  import Icon from './Icon.svelte';

  let apps = $derived(currentApps());
  let over = $state('');
  let dragging = $state('');
  const setVol = (name, v) => { S.apps.vol[name.toLowerCase()] = v; save(); };
  const toggleMute = (name) => { S.apps.mute[name.toLowerCase()] = !appMuted(name); save(); };
</script>

<div class="top">
  <div>
    <h2>Applications</h2>
    <p class="sub">
      {#if S.rt.live}{apps.length} apps are using audio on this PC. Drag an app to another source bus — its volume then follows that bus.
      {:else}Demo apps. In the Windows app this list shows the programs that are really playing sound.{/if}
    </p>
  </div>
  <button class="btn" onclick={resetAppVolumes} title="Set every app back to 100% and unmute">Reset app volumes</button>
</div>

<div class="cols">
  {#each S.mix.sources as s}
    {@const list = apps.filter((a) => assignedSource(a.name) === s.id)}
    <section class="col" class:over={over === s.id} style="--c:{s.color}" aria-label="{label(s)} apps"
      ondragover={(e) => { e.preventDefault(); over = s.id; }} ondragleave={() => (over = '')}
      ondrop={(e) => { e.preventDefault(); over = ''; const n = e.dataTransfer.getData('text/plain'); if (n) assignApp(n, s.id); }}>
      <header><span class="sicon"><Icon name={s.id} /></span><b>{label(s)}</b><small>{list.length}</small></header>
      {#each list as a (a.name + a.pid)}
        <div class="app" role="group" aria-label={a.name} class:dragging={dragging === a.name} draggable="true"
          ondragstart={(e) => { e.dataTransfer.setData('text/plain', a.name); dragging = a.name; }} ondragend={() => (dragging = '')}>
          <div class="row1">
            <span class="grip" aria-hidden="true"><Icon name="grip" size={14} /></span>
            <span class="glyph">{a.name[0]}</span>
            <span class="aname" title={a.name}>{a.name}</span>
            {#if !hasRule(a.name)}<span class="auto" title="Placed automatically. Drag it or pick a bus to save a rule.">Auto</span>{/if}
            <button class="mute" aria-pressed={appMuted(a.name)} onclick={() => toggleMute(a.name)} aria-label="Mute {a.name}">M</button>
          </div>
          <Meter key={'app:' + a.name} color={s.color} label={a.name} horizontal channels={1} />
          <div class="row2">
            <input type="range" min="0" max="100" value={Math.round(appVol(a.name) * 100)} aria-label="{a.name} volume"
              oninput={(e) => setVol(a.name, +e.currentTarget.value / 100)} />
            <span class="pct">{Math.round(appVol(a.name) * 100)}%</span>
            <select class="select mini" value={s.id} onchange={(e) => assignApp(a.name, e.currentTarget.value)} aria-label="Source bus for {a.name}">
              {#each S.mix.sources as o}<option value={o.id}>{label(o)}</option>{/each}
            </select>
          </div>
        </div>
      {:else}
        <p class="empty">Drop apps here</p>
      {/each}
    </section>
  {/each}
</div>

<style>
  .top { display: flex; align-items: flex-start; justify-content: space-between; gap: 16px; margin-bottom: 14px; }
  h2 { margin: 0 0 4px; font-size: 15px; }
  .sub { margin: 0; color: var(--muted); max-width: 640px; }
  .cols { display: grid; grid-template-columns: repeat(4, minmax(220px, 1fr)); gap: 10px; min-width: 920px; }
  .col { background: var(--strip); border: 1px solid var(--line); border-top: 3px solid var(--c); border-radius: var(--r); padding: 10px; min-height: 320px; display: flex; flex-direction: column; gap: 8px; transition: background .15s; }
  .col.over { background: color-mix(in srgb, var(--c) 10%, var(--strip)); border-color: var(--c); }
  header { display: flex; align-items: center; gap: 8px; margin-bottom: 2px; }
  header b { text-transform: uppercase; letter-spacing: .5px; font-size: 12.5px; flex: 1; }
  header small { color: var(--muted); font-variant-numeric: tabular-nums; }
  .sicon { width: 26px; height: 26px; border-radius: 8px; display: grid; place-items: center; color: var(--c); background: color-mix(in srgb, var(--c) 16%, transparent); }
  .app { background: var(--strip-2); border: 1px solid var(--line); border-radius: 9px; padding: 8px; display: flex; flex-direction: column; gap: 7px; cursor: grab; }
  .app.dragging { opacity: .4; }
  .row1, .row2 { display: flex; align-items: center; gap: 7px; }
  .grip { color: var(--dim); }
  .glyph { width: 20px; height: 20px; border-radius: 5px; display: grid; place-items: center; background: var(--c); color: #0d0f13; font-weight: 800; font-size: 11px; flex: none; }
  .aname { flex: 1; min-width: 0; font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .auto { font-size: 9.5px; font-weight: 800; letter-spacing: .4px; color: var(--warn); border: 1px solid rgba(255,209,102,.4); border-radius: 4px; padding: 0 4px; }
  .mute { width: 26px; height: 24px; border-radius: 6px; font-weight: 800; font-size: 11px; border: 1px solid var(--line-2); color: var(--muted); }
  .mute[aria-pressed="true"] { background: rgba(255,93,93,.18); border-color: rgba(255,93,93,.6); color: #ff8a8a; }
  .row2 input { flex: 1; }
  .pct { width: 36px; text-align: right; font-size: 11px; color: var(--muted); font-variant-numeric: tabular-nums; }
  .mini { height: 24px; font-size: 11px; padding: 0 4px; width: 72px; }
  .empty { color: var(--dim); font-size: 12px; text-align: center; margin: 24px 0; border: 1px dashed var(--line-2); border-radius: 9px; padding: 18px 8px; }
</style>
