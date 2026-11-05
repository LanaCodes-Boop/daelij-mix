<script>
  import { S, commit, label } from '../lib/state.svelte.js';
  import Fader from './Fader.svelte';
  import Meter from './Meter.svelte';
  import DbValue from './DbValue.svelte';
  import Icon from './Icon.svelte';
  let { mix } = $props();
  let focused = $derived(S.ui.focus === mix.id);
  let selected = $derived(S.ui.selected === 'mix:' + mix.id);
  let dense = $derived(S.ui.density);
  let name = $derived(label(mix));
  let device = $derived(
    mix.id === 'A'
      ? (S.rt.live ? (S.rt.outputs.find((o) => o.is_default)?.name ?? 'Default output') : 'Default output')
      : (S.rt.live ? 'Preview — needs virtual driver' : 'Virtual output'));
</script>

<section class="strip" class:focused class:selected class:muted={mix.mute} class:compact={dense === 'compact'}
  style="--c:{mix.color}" aria-label="{name} output">
  <button class="head" onclick={() => (S.ui.selected = 'mix:' + mix.id)} aria-pressed={selected} title="Select {name} for the inspector">
    <span class="sicon"><Icon name={mix.id} /></span>
    <span class="name">{name}</span>
  </button>
  {#if dense !== 'compact'}<div class="dev" title={device}>{mix.name} · {device}</div>{/if}
  <button class="focusbtn" aria-pressed={focused} onclick={() => (S.ui.focus = mix.id)}
    title="Show the sends to {name} on the source faders">
    {focused ? 'Sends shown' : 'Show sends'}
  </button>

  <div class="fz">
    <Fader value={mix.gain} color={mix.color} label="{name} output volume" ticks={dense !== 'compact'}
      onchange={(db) => (mix.gain = db)} oncommit={commit}>
      <Meter key={'mix:' + mix.id} color={mix.color} label={name} />
    </Fader>
  </div>
  <div class="readout"><DbValue value={mix.gain} label="{name} output" onset={(v) => { mix.gain = v; commit(); }} /></div>
  <div class="ms">
    <button class="m" aria-pressed={mix.mute} onclick={() => { mix.mute = !mix.mute; commit(); }} aria-label="Mute {name}">MUTE</button>
  </div>
</section>

<style>
  .strip { position: relative; flex: 1 1 0; min-width: 132px; max-width: 190px; display: flex; flex-direction: column;
    background: linear-gradient(180deg, color-mix(in srgb, var(--c) 7%, var(--strip-2)), var(--strip) 160px);
    border: 1px solid var(--line); border-radius: var(--r); overflow: hidden; }
  .strip.compact { min-width: 96px; max-width: 130px; }
  .strip::before { content: ""; position: absolute; inset: 0 0 auto; height: 3px; background: var(--c); }
  .strip.focused { background: linear-gradient(180deg, color-mix(in srgb, var(--c) 16%, var(--strip-2)), var(--strip) 200px); }
  .strip.selected { border-color: color-mix(in srgb, var(--c) 70%, transparent); box-shadow: 0 0 0 1px color-mix(in srgb, var(--c) 45%, transparent); }
  .strip.muted .fz { opacity: .45; }
  .strip:has(:global(.meter[data-clip]))::after { content: "CLIP · press R"; position: absolute; top: 6px; right: 6px; font-size: 9px; font-weight: 800; color: #fff; background: var(--hot); padding: 1px 5px; border-radius: 4px; }
  .head { display: flex; align-items: center; gap: 7px; padding: 12px 10px 4px; text-align: left; width: 100%; }
  .sicon { width: 26px; height: 26px; border-radius: 8px; display: grid; place-items: center; color: var(--c); background: color-mix(in srgb, var(--c) 18%, transparent); flex: none; }
  .compact .sicon { display: none; }
  .name { font-weight: 700; font-size: 12.5px; letter-spacing: .5px; text-transform: uppercase; flex: 1; min-width: 0; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .dev { margin: 0 10px; font-size: 11px; color: var(--muted); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .focusbtn { margin: 8px 10px 0; height: 28px; border-radius: 7px; font-size: 11.5px; font-weight: 700; border: 1px solid var(--line-2); color: var(--muted); }
  .focusbtn[aria-pressed="true"] { background: var(--c); color: #0d0f13; border-color: transparent; }
  .fz { flex: 1; min-height: 140px; }
  .readout { display: flex; justify-content: center; padding: 0 8px; }
  .ms { display: flex; padding: 8px 10px 10px; }
  .ms button { flex: 1; height: 30px; border-radius: 7px; font-weight: 800; font-size: 11.5px; letter-spacing: .5px; border: 1px solid var(--line-2); color: var(--muted); background: var(--strip-2); }
  .ms .m[aria-pressed="true"] { background: rgba(255,93,93,.18); border-color: rgba(255,93,93,.6); color: #ff8a8a; }
</style>
