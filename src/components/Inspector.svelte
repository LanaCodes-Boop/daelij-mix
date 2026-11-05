<script>
  import { S, commit, setSendGain, label, source, mixOf, EQ_PRESETS, bandsFor, assignedSource } from '../lib/state.svelte.js';
  import { currentApps } from '../lib/engine.js';
  import DbValue from './DbValue.svelte';
  import EqCurve from './EqCurve.svelte';
  import Icon from './Icon.svelte';
  import { dbToPos, posToDb } from '../lib/state.svelte.js';

  let isMix = $derived(S.ui.selected.startsWith('mix:'));
  let src = $derived(isMix ? null : source(S.ui.selected));
  let mix = $derived(isMix ? mixOf(S.ui.selected.slice(4)) : null);
  let item = $derived(src ?? mix);
  let detailed = $derived(S.ui.density === 'detailed');
  let apps = $derived(src ? currentApps().filter((a) => assignedSource(a.name) === src.id) : []);
  const sendersTo = (m) => S.mix.sources.filter((s) => S.mix.sends[s.id][m].on);
</script>

{#if item}
<aside class="insp" style="--c:{item.color}" aria-label="Inspector">
  <header>
    <span class="sicon"><Icon name={isMix ? mix.id : src.id} /></span>
    <div><h3>{label(item)}</h3><small>{isMix ? 'Output · ' + item.name : 'Source bus'}</small></div>
    <button class="iconbtn" onclick={() => (S.ui.inspector = false)} aria-label="Close inspector"><Icon name="close" size={14} /></button>
  </header>

  <div class="body">
    <section>
      <label class="field"><span>Alias</span>
        <input class="text" value={item.alias} placeholder={item.name}
          onchange={(e) => { item.alias = e.currentTarget.value.trim(); commit(); }} aria-label="Alias for {item.name}" />
      </label>
    </section>

    {#if src}
      <section>
        <h4>Routing</h4>
        {#each S.mix.mixes as m}
          {@const sd = S.mix.sends[src.id][m.id]}
          <div class="route" style="--m:{m.color}">
            <button class="switch" style="--c:{m.color}" aria-pressed={sd.on} aria-label="Send {label(src)} to {label(m)}"
              onclick={() => { sd.on = !sd.on; commit(); }}></button>
            <span class="rname">{label(m)}</span>
            <input type="range" min="0" max="1000" value={Math.round(dbToPos(sd.gain) * 1000)} aria-label="{label(src)} send to {label(m)}"
              oninput={(e) => setSendGain(src.id, m.id, posToDb(+e.currentTarget.value / 1000))} onchange={commit} />
            <DbValue value={sd.gain} label="{label(src)} to {label(m)}" onset={(v) => { setSendGain(src.id, m.id, v); commit(); }} />
          </div>
        {/each}
      </section>

      <section>
        <h4>Group <small>faders in a group move together</small></h4>
        <div class="seg" role="group" aria-label="Group">
          {#each [0, 1, 2, 3] as g}
            <button aria-pressed={src.group === g} onclick={() => { src.group = g; commit(); }}>{g ? 'Group ' + g : 'None'}</button>
          {/each}
        </div>
      </section>

      <section>
        <h4>Equalizer
          <button class="switch" aria-pressed={src.eqOn !== false} aria-label="Equalizer on"
            onclick={() => { src.eqOn = src.eqOn === false; commit(); }}></button></h4>
        <div class="presets">
          {#each Object.keys(EQ_PRESETS) as p}
            <button aria-pressed={src.preset === p} onclick={() => { src.preset = p; src.eq = bandsFor(p); commit(); }}>{p}</button>
          {/each}
        </div>
        <div class:dimmed={src.eqOn === false}>
          <EqCurve bands={src.eq} color={src.color} onchange={() => (src.preset = 'Custom')} oncommit={commit} />
        </div>
        <p class="hint">Drag a point · scroll on it for width · double-click resets · arrow keys work too.</p>
      </section>

      <section>
        <h4>Processing</h4>
        {#each [['comp', 'Compressor', 'Evens out loud and quiet parts', 'amount', 0, 100, '%'],
                ['gate', 'Noise gate', 'Silences quiet background sound', 'thr', -80, -10, 'dB'],
                ['nr', 'Noise reduction', 'Removes hum, fans and hiss', 'amount', 0, 100, '%']] as [k, t, d, p, lo, hi, u]}
          <div class="proc" class:on={src[k].on}>
            <div class="ptop"><div><b>{t}</b><small>{d}</small></div>
              <button class="switch" aria-pressed={src[k].on} aria-label={t} onclick={() => { src[k].on = !src[k].on; commit(); }}></button></div>
            {#if detailed}
              <label class="pr"><span>{u === 'dB' ? 'Threshold' : 'Amount'}</span>
                <input type="range" min={lo} max={hi} bind:value={src[k][p]} onchange={commit} aria-label="{t} {u === 'dB' ? 'threshold' : 'amount'}" />
                <span class="v">{src[k][p]} {u}</span></label>
            {/if}
          </div>
        {/each}
        {#if !detailed}<p class="hint">Switch to Detailed for all processing controls.</p>{/if}
      </section>

      <section>
        <h4>Apps on {label(src)}</h4>
        {#if apps.length}<ul class="applist">{#each apps as a}<li>{a.name}</li>{/each}</ul>
        {:else}<p class="hint">No apps here. Assign them in Applications.</p>{/if}
      </section>
    {:else}
      <section>
        <h4>Feeding this mix</h4>
        {#each sendersTo(mix.id) as s}
          {@const sd = S.mix.sends[s.id][mix.id]}
          <div class="route" style="--m:{s.color}">
            <span class="dot" style="background:{s.color}"></span><span class="rname">{label(s)}</span>
            <input type="range" min="0" max="1000" value={Math.round(dbToPos(sd.gain) * 1000)} aria-label="{label(s)} send to {label(mix)}"
              oninput={(e) => setSendGain(s.id, mix.id, posToDb(+e.currentTarget.value / 1000))} onchange={commit} />
            <DbValue value={sd.gain} label="{label(s)} to {label(mix)}" onset={(v) => { setSendGain(s.id, mix.id, v); commit(); }} />
          </div>
        {:else}<p class="hint">Nothing is sent here yet.</p>{/each}
      </section>
      <section>
        <h4>Device</h4>
        {#if mix.id === 'A'}
          <p class="hint">{S.rt.live ? 'Plays on your Windows default output. Change it in the device menu at the top.' : 'In the Windows app this is your default output device.'}</p>
        {:else}
          <p class="hint">Mix B is the stream mix. Daelij MiX records each app separately and plays this mix on your stream device — best a virtual cable (free VB-Cable). In OBS add "Audio Input Capture" and pick "CABLE Output".</p>
          {#if S.rt.live}
            <p class="hint">Now: {S.rt.stream.state === 'active' || S.rt.stream.state === 'fallback' ? `playing on ${S.rt.stream.device} · ${S.rt.stream.captures} apps` : S.rt.stream.state.replace('_', ' ')}{S.rt.stream.error ? ' · ' + S.rt.stream.error : ''}</p>
          {/if}
          <p class="hint">Pick and order stream devices in the device menu at the top right.</p>
        {/if}
      </section>
    {/if}
  </div>
</aside>
{/if}

<style>
  .insp { width: 330px; flex: none; display: flex; flex-direction: column; background: var(--surface); border: 1px solid var(--line); border-radius: var(--r); overflow: hidden; min-height: 0; }
  header { display: flex; align-items: center; gap: 10px; padding: 12px 10px 12px 14px; border-bottom: 1px solid var(--line); border-top: 3px solid var(--c); }
  header div { flex: 1; min-width: 0; }
  h3 { margin: 0; font-size: 14px; text-transform: uppercase; letter-spacing: .6px; }
  header small { color: var(--muted); font-size: 11px; }
  .sicon { width: 28px; height: 28px; border-radius: 8px; display: grid; place-items: center; color: var(--c); background: color-mix(in srgb, var(--c) 16%, transparent); }
  .body { overflow: auto; padding: 4px 14px 16px; }
  section { padding: 12px 0; border-bottom: 1px solid var(--line); }
  section:last-child { border-bottom: 0; }
  h4 { margin: 0 0 10px; font-size: 12px; display: flex; align-items: center; gap: 8px; justify-content: space-between; }
  h4 small { color: var(--dim); font-weight: 500; margin-right: auto; }
  .field { display: flex; align-items: center; gap: 10px; font-size: 12px; color: var(--muted); }
  .text { flex: 1; height: 30px; border-radius: 8px; background: var(--well); border: 1px solid var(--line-2); padding: 0 9px; }
  .route { display: flex; align-items: center; gap: 8px; margin-bottom: 8px; }
  .route input[type=range] { flex: 1; }
  .rname { width: 76px; font-size: 12px; font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .dot { width: 8px; height: 8px; border-radius: 50%; flex: none; }
  .route :global(.dbv), .route :global(.dbin) { min-width: 62px; width: 62px; font-size: 11.5px; padding: 3px 4px; }
  .seg button { flex: 1; padding: 4px 6px; }
  .presets { display: flex; flex-wrap: wrap; gap: 5px; margin-bottom: 8px; }
  .presets button { padding: 3px 9px; border-radius: 7px; font-size: 11.5px; border: 1px solid var(--line-2); color: var(--muted); }
  .presets button[aria-pressed="true"] { background: var(--c); color: #0d0f13; border-color: transparent; font-weight: 700; }
  .dimmed { opacity: .35; }
  .hint { font-size: 11px; color: var(--dim); margin: 6px 0 0; }
  .proc { background: var(--strip); border: 1px solid var(--line); border-radius: 9px; padding: 9px 10px; margin-bottom: 6px; }
  .ptop { display: flex; align-items: center; justify-content: space-between; gap: 8px; }
  .ptop b { font-size: 12px; display: block; }
  .ptop small { color: var(--muted); font-size: 10.5px; }
  .proc:not(.on) .pr { opacity: .4; }
  .pr { display: flex; align-items: center; gap: 8px; font-size: 11px; color: var(--muted); margin-top: 8px; }
  .pr input { flex: 1; }
  .pr .v { width: 52px; text-align: right; color: var(--text); font-variant-numeric: tabular-nums; }
  .applist { margin: 0; padding-left: 16px; color: var(--muted); font-size: 12px; }
</style>
