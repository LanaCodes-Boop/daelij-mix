<script>
  import { S, commit, setSendGain, label, dbToPos, posToDb } from '../lib/state.svelte.js';
  import DbValue from './DbValue.svelte';
  import Icon from './Icon.svelte';
</script>

<div class="wrap">
  <div class="card">
    <h2>Matrix</h2>
    <p class="sub">Every source against every mix. Switch a route on or off and set how loud it is sent.</p>
    <div class="scroll">
      <table>
        <thead><tr><th></th>{#each S.mix.mixes as m}<th style="color:{m.color}">{label(m)}<small>{m.name}</small></th>{/each}</tr></thead>
        <tbody>
          {#each S.mix.sources as s}
            <tr>
              <th scope="row"><span class="sicon" style="--c:{s.color}"><Icon name={s.id} /></span>{label(s)}</th>
              {#each S.mix.mixes as m}
                {@const sd = S.mix.sends[s.id][m.id]}
                <td>
                  <div class="cell" class:on={sd.on} style="--c:{s.color}">
                    <button class="tog" aria-pressed={sd.on} onclick={() => { sd.on = !sd.on; commit(); }}
                      aria-label="Route {label(s)} to {label(m)}">{sd.on ? 'On' : 'Off'}</button>
                    <input type="range" min="0" max="1000" value={Math.round(dbToPos(sd.gain) * 1000)}
                      aria-label="{label(s)} send to {label(m)}"
                      oninput={(e) => setSendGain(s.id, m.id, posToDb(+e.currentTarget.value / 1000))} onchange={commit} />
                    <DbValue value={sd.gain} label="{label(s)} to {label(m)}" onset={(v) => { setSendGain(s.id, m.id, v); commit(); }} />
                  </div>
                </td>
              {/each}
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  </div>
</div>

<style>
  .wrap { max-width: 980px; }
  .scroll { overflow-x: auto; }
  table { border-collapse: separate; border-spacing: 8px; }
  thead th { font-size: 12px; text-align: left; padding: 0 4px; text-transform: uppercase; letter-spacing: .6px; }
  thead small { display: block; color: var(--dim); text-transform: none; letter-spacing: 0; font-weight: 500; }
  tbody th { text-align: left; font-weight: 700; text-transform: uppercase; letter-spacing: .5px; white-space: nowrap; padding-right: 10px; }
  tbody th .sicon { display: inline-grid; vertical-align: middle; margin-right: 10px; width: 26px; height: 26px; border-radius: 8px; place-items: center; color: var(--c); background: color-mix(in srgb, var(--c) 16%, transparent); }
  .cell { display: flex; align-items: center; gap: 10px; padding: 8px 10px; border-radius: 10px; border: 1px solid var(--line-2); background: var(--well); min-width: 330px; }
  .cell.on { border-color: color-mix(in srgb, var(--c) 45%, transparent); background: color-mix(in srgb, var(--c) 7%, var(--well)); }
  .cell:not(.on) input, .cell:not(.on) :global(.dbv) { opacity: .4; }
  .tog { width: 48px; height: 28px; border-radius: 7px; font-weight: 700; font-size: 12px; border: 1px solid var(--line-2); color: var(--dim); flex: none; }
  .tog[aria-pressed="true"] { background: var(--c); color: #0d0f13; border-color: transparent; }
  .cell input { flex: 1; }
</style>
