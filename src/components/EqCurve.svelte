<script>
  // SVG equalizer: drag a point (frequency + gain), scroll for width (Q), double-click to reset.
  let { bands, color = '#8b9cff', onchange, oncommit } = $props();
  const W = 340, H = 150, FMIN = 20, FMAX = 20000, G = 15;
  const fx = (f) => (Math.log(f / FMIN) / Math.log(FMAX / FMIN)) * W;
  const xf = (x) => FMIN * Math.pow(FMAX / FMIN, x / W);
  const gy = (g) => H / 2 - (g / G) * (H / 2 - 10);
  const yg = (y) => Math.max(-12, Math.min(12, ((H / 2 - y) / (H / 2 - 10)) * G));
  function resp(b, f) {
    const o = Math.log2(f / b.f);
    if (b.type === 'low') return b.g / (1 + Math.exp(o * 3.2 * b.q));
    if (b.type === 'high') return b.g / (1 + Math.exp(-o * 3.2 * b.q));
    const s = 0.75 / b.q; return b.g * Math.exp(-(o * o) / (2 * s * s));
  }
  let path = $derived.by(() => {
    let d = '';
    for (let x = 0; x <= W; x += 3) {
      const f = xf(x), g = bands.reduce((s, b) => s + resp(b, f), 0);
      d += (x ? 'L' : 'M') + x + ' ' + gy(Math.max(-G, Math.min(G, g))).toFixed(1);
    }
    return d;
  });
  let svgEl, drag = $state(-1);
  function local(e) { const r = svgEl.getBoundingClientRect(); return [((e.clientX - r.left) / r.width) * W, ((e.clientY - r.top) / r.height) * H]; }
  function down(e, i) { drag = i; svgEl.setPointerCapture(e.pointerId); e.preventDefault(); }
  function move(e) {
    if (drag < 0) return;
    const [x, y] = local(e), b = bands[drag];
    b.f = Math.max(25, Math.min(18000, xf(Math.max(0, Math.min(W, x)))));
    b.g = Math.round(yg(y) * 2) / 2; onchange();
  }
  function up() { if (drag >= 0) { drag = -1; oncommit(); } }
  function wheel(e, i) { e.preventDefault(); const b = bands[i]; b.q = Math.max(0.3, Math.min(4, b.q * (e.deltaY < 0 ? 1.12 : 1 / 1.12))); onchange(); oncommit(); }
  function key(e, i) {
    const b = bands[i];
    if (e.key === 'ArrowUp') b.g = Math.min(12, b.g + 0.5);
    else if (e.key === 'ArrowDown') b.g = Math.max(-12, b.g - 0.5);
    else if (e.key === 'ArrowRight') b.f = Math.min(18000, b.f * 1.06);
    else if (e.key === 'ArrowLeft') b.f = Math.max(25, b.f / 1.06);
    else if (e.key === 'Delete' || e.key === 'Backspace') { b.g = 0; b.q = 1; }
    else return;
    e.preventDefault(); onchange(); oncommit();
  }
  const fl = (f) => (f >= 1000 ? (f / 1000).toFixed(1) + ' kHz' : Math.round(f) + ' Hz');
</script>

<svg bind:this={svgEl} viewBox="0 0 {W} {H}" class="eq" style="--c:{color}" onpointermove={move} onpointerup={up} onpointercancel={up} role="group" aria-label="Equalizer">
  {#each [50, 100, 200, 500, 1000, 2000, 5000, 10000] as f}
    <line x1={fx(f)} x2={fx(f)} y1="0" y2={H} class="grid" />
    <text x={fx(f) + 3} y={H - 4} class="lbl">{f >= 1000 ? f / 1000 + 'k' : f}</text>
  {/each}
  {#each [-12, -6, 6, 12] as g}<line x1="0" x2={W} y1={gy(g)} y2={gy(g)} class="grid" />{/each}
  <line x1="0" x2={W} y1={gy(0)} y2={gy(0)} class="zero" />
  <path d="{path} L{W} {gy(0)} L0 {gy(0)} Z" class="area" />
  <path d={path} class="curve" />
  {#each bands as b, i}
    <g class="node" class:active={drag === i} transform="translate({fx(b.f)} {gy(b.g)})"
      role="slider" tabindex="0" aria-label="Band {i + 1}, {fl(b.f)}" aria-valuemin="-12" aria-valuemax="12" aria-valuenow={b.g}
      aria-valuetext="{b.g > 0 ? '+' : ''}{b.g} dB at {fl(b.f)}"
      onpointerdown={(e) => down(e, i)} ondblclick={() => { b.g = 0; b.q = 1; onchange(); oncommit(); }}
      onwheel={(e) => wheel(e, i)} onkeydown={(e) => key(e, i)}>
      <circle r="8" /><text y="3.5">{i + 1}</text>
    </g>
  {/each}
  {#if drag >= 0}<text x="8" y="16" class="tip">{fl(bands[drag].f)} · {bands[drag].g > 0 ? '+' : ''}{bands[drag].g} dB</text>{/if}
</svg>

<style>
  .eq { width: 100%; height: auto; display: block; background: var(--well); border-radius: 8px; touch-action: none; }
  .grid { stroke: rgba(255,255,255,.05); }
  .zero { stroke: rgba(255,255,255,.15); }
  .lbl { fill: rgba(255,255,255,.3); font-size: 8.5px; }
  .area { fill: color-mix(in srgb, var(--c) 14%, transparent); }
  .curve { fill: none; stroke: var(--c); stroke-width: 2; }
  .node { cursor: grab; outline: none; }
  .node circle { fill: #0d0f13; stroke: var(--c); stroke-width: 2; }
  .node text { fill: #fff; font-size: 9px; font-weight: 700; text-anchor: middle; pointer-events: none; }
  .node.active circle, .node:focus-visible circle { fill: var(--c); }
  .node.active text, .node:focus-visible text { fill: #0d0f13; }
  .tip { fill: rgba(255,255,255,.8); font-size: 10px; font-weight: 600; }
</style>
