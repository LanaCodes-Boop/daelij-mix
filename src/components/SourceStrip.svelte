<script>
  import { S, commit, setSendGain, label, mixOf, assignedSource } from '../lib/state.svelte.js';
  import { currentApps } from '../lib/engine.js';
  import Fader from './Fader.svelte';
  import Meter from './Meter.svelte';
  import DbValue from './DbValue.svelte';
  import Icon from './Icon.svelte';
  let { src } = $props();

  let focus = $derived(S.ui.focus);
  let send = $derived(S.mix.sends[src.id][focus]);
  let fmix = $derived(mixOf(focus));
  let selected = $derived(S.ui.selected === src.id);
  let dense = $derived(S.ui.density);
  let apps = $derived(currentApps().filter((a) => assignedSource(a.name) === src.id));
  let name = $derived(label(src));
  const toggle = (fn) => { fn(); commit(); };
</script>

<section class="strip" class:selected class:muted={src.mute} class:off={!send.on}
  class:compact={dense === 'compact'} style="--c:{src.color}" aria-label="{name} strip">
  <button class="head" onclick={() => (S.ui.selected = src.id)} aria-pressed={selected}
    title="Select {name} to edit it in the inspector">
    <span class="sicon"><Icon name={src.id} /></span>
    <span class="name">{name}</span>
    {#if src.group}<span class="grp" title="Group {src.group}">G{src.group}</span>{/if}
  </button>

  {#if dense !== 'compact'}
    <div class="apps" title={apps.map((a) => a.name).join(', ')}>
      {#if apps.length}
        {apps.slice(0, 2).map((a) => a.name).join(', ')}{apps.length > 2 ? ` +${apps.length - 2}` : ''}
      {:else}<span class="dim">No apps</span>{/if}
    </div>
    <div class="sends" role="group" aria-label="{name} sends">
      {#each S.mix.mixes as m}
        {@const sd = S.mix.sends[src.id][m.id]}
        <button class="send" class:focus={m.id === focus} style="--m:{m.color}" aria-pressed={sd.on}
          onclick={() => toggle(() => (sd.on = !sd.on))}
          title="{sd.on ? 'Stop sending' : 'Send'} {name} to {label(m)}">
          <i></i>{m.id}<b>{sd.gain <= -60 ? '−∞' : sd.gain.toFixed(1)}</b>
        </button>
      {/each}
    </div>
    <div class="proc" role="group" aria-label="{name} processing">
      <button aria-pressed={src.eqOn !== false} onclick={() => toggle(() => (src.eqOn = src.eqOn === false))}>EQ</button>
      <button aria-pressed={src.comp.on} onclick={() => toggle(() => (src.comp.on = !src.comp.on))}>COMP</button>
      <button aria-pressed={src.gate.on} onclick={() => toggle(() => (src.gate.on = !src.gate.on))}>GATE</button>
    </div>
  {/if}

  <div class="fz">
    <Fader value={send.gain} color={src.color} label="{name} send to {label(fmix)}" ticks={dense !== 'compact'}
      onchange={(db) => setSendGain(src.id, focus, db)} oncommit={commit}>
      <Meter key={'src:' + src.id} color={src.color} label={name} />
    </Fader>
  </div>

  <div class="readout">
    <DbValue value={send.gain} label="{name} to {label(fmix)}" onset={(v) => { setSendGain(src.id, focus, v); commit(); }} />
  </div>
  <div class="to" style="--m:{fmix.color}">→ {label(fmix)}</div>

  {#if dense !== 'compact'}
    <label class="pan"><span>L</span>
      <input type="range" min="-100" max="100" bind:value={src.pan} onchange={commit}
        ondblclick={() => { src.pan = 0; commit(); }} aria-label="{name} balance" /><span>R</span></label>
  {/if}

  <div class="ms">
    <button class="m" aria-pressed={src.mute} onclick={() => toggle(() => (src.mute = !src.mute))} aria-label="Mute {name}">M</button>
    <button class="s" aria-pressed={src.solo} onclick={() => toggle(() => (src.solo = !src.solo))} aria-label="Solo {name}">S</button>
  </div>
</section>

<style>
  .strip { position: relative; flex: 1 1 0; min-width: 132px; max-width: 190px; display: flex; flex-direction: column;
    background: linear-gradient(180deg, color-mix(in srgb, var(--c) 6%, var(--strip-2)), var(--strip) 160px);
    border: 1px solid var(--line); border-radius: var(--r); overflow: hidden; }
  .strip.compact { min-width: 86px; max-width: 120px; }
  .strip::before { content: ""; position: absolute; inset: 0 0 auto; height: 3px; background: var(--c); opacity: .85; }
  .strip.selected { border-color: color-mix(in srgb, var(--c) 70%, transparent); box-shadow: 0 0 0 1px color-mix(in srgb, var(--c) 45%, transparent); }
  .strip.muted .fz, .strip.off .fz { opacity: .45; }
  .strip:has(:global(.meter[data-clip]))::after { content: "CLIP · press R"; position: absolute; top: 6px; right: 6px; font-size: 9px; font-weight: 800; letter-spacing: .5px; color: #fff; background: var(--hot); padding: 1px 5px; border-radius: 4px; }
  .head { display: flex; align-items: center; gap: 7px; padding: 12px 10px 6px; text-align: left; width: 100%; }
  .sicon { width: 26px; height: 26px; border-radius: 8px; display: grid; place-items: center; color: var(--c); background: color-mix(in srgb, var(--c) 16%, transparent); flex: none; }
  .name { font-weight: 700; font-size: 12.5px; letter-spacing: .5px; text-transform: uppercase; flex: 1; min-width: 0; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .compact .sicon { display: none; }
  .grp { font-size: 9.5px; font-weight: 800; padding: 1px 5px; border-radius: 4px; background: var(--well); color: var(--muted); }
  .apps { margin: 0 10px; font-size: 11px; color: var(--muted); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .dim { color: var(--dim); }
  .sends { display: flex; flex-direction: column; gap: 3px; margin: 8px 10px 0; }
  .send { display: flex; align-items: center; gap: 6px; height: 22px; padding: 0 7px; border-radius: 6px; font-size: 11px; font-weight: 700; color: var(--dim); background: var(--well); border: 1px solid transparent; }
  .send i { width: 7px; height: 7px; border-radius: 50%; background: var(--dim); }
  .send b { margin-left: auto; font-variant-numeric: tabular-nums; font-weight: 600; }
  .send[aria-pressed="true"] { color: var(--text); }
  .send[aria-pressed="true"] i { background: var(--m); }
  .send.focus { border-color: color-mix(in srgb, var(--m) 55%, transparent); }
  .proc { display: flex; gap: 3px; margin: 6px 10px 0; }
  .proc button { flex: 1; height: 20px; border-radius: 5px; font-size: 9.5px; font-weight: 800; letter-spacing: .4px; color: var(--dim); border: 1px solid var(--line-2); }
  .proc button[aria-pressed="true"] { color: var(--c); border-color: color-mix(in srgb, var(--c) 50%, transparent); background: color-mix(in srgb, var(--c) 10%, transparent); }
  .fz { flex: 1; min-height: 140px; }
  .readout { display: flex; justify-content: center; padding: 0 8px; }
  .to { text-align: center; font-size: 10.5px; color: var(--m); margin-top: 3px; font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; padding: 0 6px; }
  .pan { display: flex; align-items: center; gap: 6px; padding: 6px 12px 0; font-size: 10px; color: var(--dim); font-weight: 700; }
  .pan input { flex: 1; }
  .ms { display: flex; gap: 5px; padding: 8px 10px 10px; }
  .ms button { flex: 1; height: 30px; border-radius: 7px; font-weight: 800; font-size: 12px; border: 1px solid var(--line-2); color: var(--muted); background: var(--strip-2); }
  .ms .m[aria-pressed="true"] { background: rgba(255,93,93,.18); border-color: rgba(255,93,93,.6); color: #ff8a8a; }
  .ms .s[aria-pressed="true"] { background: rgba(255,209,102,.18); border-color: rgba(255,209,102,.6); color: var(--warn); }
</style>
