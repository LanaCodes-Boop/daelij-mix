<script>
  import { dbToPos, posToDb, fmtDb, clampDb } from '../lib/state.svelte.js';
  let { value, color = '#8b9cff', label = 'Volume', onchange, oncommit, ticks = true, children } = $props();
  const TICKS = [12, 6, 0, -6, -12, -24, -40];
  let rail;
  let drag = null;
  let pos = $derived(dbToPos(value));

  function posAt(y) { const r = rail.getBoundingClientRect(); return Math.min(1, Math.max(0, 1 - (y - r.top) / r.height)); }
  function down(e) {
    if (e.button !== 0) return;
    rail.setPointerCapture(e.pointerId);
    let p = pos;
    if (!e.target.closest('.thumb')) { p = posAt(e.clientY); onchange(posToDb(p)); }
    drag = { y: e.clientY, p, h: rail.getBoundingClientRect().height };
    rail.querySelector('.thumb').focus({ preventScroll: true });
  }
  function move(e) {
    if (!drag) return;
    const f = e.shiftKey ? 0.12 : 1;               // Shift = fine
    drag.p = Math.min(1, Math.max(0, drag.p - ((e.clientY - drag.y) / drag.h) * f));
    drag.y = e.clientY;
    onchange(posToDb(drag.p));
  }
  function up() { if (drag) { drag = null; oncommit(); } }
  function nudge(dir, big = false) {
    let db = value <= -60 ? -60 : value;
    const step = big ? 6 : db < -24 ? 2 : 0.5;
    db = db + dir * step;
    onchange(db < -60 ? -100 : clampDb(db));
  }
  let wheelT;
  $effect(() => {
    const w = (e) => { e.preventDefault(); nudge(e.deltaY < 0 ? 1 : -1, false); clearTimeout(wheelT); wheelT = setTimeout(oncommit, 400); };
    rail.addEventListener('wheel', w, { passive: false });
    return () => rail.removeEventListener('wheel', w);
  });
  function key(e) {
    const k = e.key;
    if (k === 'ArrowUp' || k === 'ArrowRight') nudge(1);
    else if (k === 'ArrowDown' || k === 'ArrowLeft') nudge(-1);
    else if (k === 'PageUp') nudge(1, true);
    else if (k === 'PageDown') nudge(-1, true);
    else if (k === 'Home') onchange(0);
    else if (k === 'End') onchange(-100);
    else return;
    e.preventDefault(); oncommit();
  }
</script>

<div class="fader" style="--c:{color}">
  {#if ticks}
    <div class="scale" aria-hidden="true">
      {#each TICKS as t}<span style="bottom:{dbToPos(t) * 100}%">{t > 0 ? '+' : ''}{t}</span>{/each}
      <span style="bottom:0">−∞</span>
    </div>
  {/if}
  <div class="rail" bind:this={rail} role="presentation"
    onpointerdown={down} onpointermove={move} onpointerup={up} onpointercancel={up}
    ondblclick={() => { onchange(0); oncommit(); }}>
    <div class="groove"></div>
    <div class="zero"></div>
    <div class="fill" style="height:{pos * 100}%"></div>
    <div class="thumb" role="slider" tabindex="0" aria-label={label}
      aria-valuemin="-60" aria-valuemax="12" aria-valuenow={Math.max(-60, value).toFixed(1)}
      aria-valuetext="{fmtDb(value)} dB" onkeydown={key} style="bottom:{pos * 100}%"></div>
  </div>
  {@render children?.()}
</div>

<style>
  .fader { display: flex; justify-content: center; gap: 5px; height: 100%; padding: 24px 0 22px; }
  .scale { position: relative; width: 22px; font-size: 9.5px; color: var(--dim); font-variant-numeric: tabular-nums; }
  .scale span { position: absolute; right: 0; transform: translateY(50%); }
  .rail { position: relative; width: 32px; touch-action: none; cursor: ns-resize; }
  .groove { position: absolute; left: 50%; top: 0; bottom: 0; width: 4px; transform: translateX(-50%); background: var(--well); border-radius: 3px; }
  .zero { position: absolute; left: 3px; right: 3px; bottom: 75%; height: 1px; background: var(--line-2); }
  .fill { position: absolute; left: 50%; bottom: 0; width: 4px; transform: translateX(-50%); border-radius: 3px; background: var(--c); opacity: .7; }
  .thumb { position: absolute; left: 50%; width: 30px; height: 44px; transform: translate(-50%, 50%); border-radius: 6px; cursor: grab;
    background: linear-gradient(180deg, #f3f5f9, #c9cfda 48%, #b7bfcc 52%, #dfe3ea); box-shadow: 0 4px 10px rgba(0,0,0,.45), 0 0 0 1px rgba(0,0,0,.35); }
  .thumb::before { content: ""; position: absolute; left: 4px; right: 4px; top: 50%; height: 3px; margin-top: -1.5px; border-radius: 2px; background: var(--c); }
  .thumb::after { content: ""; position: absolute; left: 9px; right: 9px; top: 9px; bottom: 9px;
    background: repeating-linear-gradient(180deg, rgba(0,0,0,.18) 0 1px, transparent 1px 4px);
    -webkit-mask: linear-gradient(#000 0 38%, transparent 38% 62%, #000 62%); mask: linear-gradient(#000 0 38%, transparent 38% 62%, #000 62%); }
  .thumb:focus-visible { outline: none; box-shadow: 0 0 0 2px var(--c), 0 4px 10px rgba(0,0,0,.45); }
</style>
