<script>
  import { S, commit, defaultMix, save } from '../lib/state.svelte.js';
  import { isTauri } from '../lib/engine.js';
  import { isVirtual } from '../lib/state.svelte.js';
  let hasCable = $derived(S.rt.outputs.some((o) => isVirtual(o.name)));
  function resetMix() { S.mix = defaultMix(); commit(); }
  function delScene(name) { if (S.scenes.list.length > 1) { S.scenes.list = S.scenes.list.filter((s) => s.name !== name); if (S.scenes.current === name) S.scenes.current = S.scenes.list[0].name; save(); } }
</script>

<div class="grid">
  <div class="card">
    <h2>Engine</h2>
    <p class="sub">Used for the latency shown at the bottom. A lower buffer means less delay but more CPU.</p>
    <div class="f"><span>Sample rate</span><div class="seg">
      {#each [44100, 48000, 96000] as v}<button aria-pressed={S.settings.sr === v} onclick={() => { S.settings.sr = v; save(); }}>{v / 1000} kHz</button>{/each}
    </div></div>
    <div class="f"><span>Buffer (samples)</span><div class="seg">
      {#each [128, 256, 512, 1024] as v}<button aria-pressed={S.settings.buf === v} onclick={() => { S.settings.buf = v; save(); }}>{v}</button>{/each}
    </div></div>
  </div>

  <div class="card">
    <h2>Stream mix (Mix B)</h2>
    <p class="sub">Plays Mix B on a separate device, so OBS gets a different mix than your headphones.</p>
    <div class="f"><span>Stream mix on</span>
      <button class="switch" aria-pressed={S.settings.streamOn} aria-label="Stream mix on" onclick={() => { S.settings.streamOn = !S.settings.streamOn; save(); }}></button></div>
    <ol class="steps">
      <li class:done={hasCable}>Install the free <b>VB-Cable</b> from vb-audio.com/Cable and restart the PC. {hasCable ? '✓ found' : ''}</li>
      <li>In Daelij MiX: device menu (top right) → Mix B → make sure "CABLE Input" is first.</li>
      <li>In OBS: Sources → + → <b>Audio Input Capture</b> → device <b>CABLE Output</b>. Mute OBS's desktop audio so nothing is double.</li>
    </ol>
    <p class="sub small">Needs Windows 11 (or Windows 10 build 20348+).</p>
  </div>

  <div class="card">
    <h2>Scenes</h2>
    <p class="sub">Saved mixer setups. Pick one from the menu at the top left.</p>
    {#each S.scenes.list as sc}
      <div class="scene"><span>{sc.name}{sc.name === S.scenes.current ? ' · active' : ''}</span>
        <button class="btn ghost" disabled={S.scenes.list.length < 2} onclick={() => delScene(sc.name)}>Delete</button></div>
    {/each}
  </div>

  <div class="card">
    <h2>Reset</h2>
    <p class="sub">Put every fader, route and effect back to the start values. You can undo this with Ctrl+Z.</p>
    <button class="btn" onclick={resetMix}>Reset mixer</button>
  </div>

  <div class="card">
    <h2>About this test version</h2>
    <p class="sub">Daelij MiX 0.2 · {isTauri ? 'Windows app' : 'web demo'}</p>
    <ul>
      <li>Game, Chat, Media and Aux control the real volume of the apps on them (Windows app).</li>
      <li>Meters follow the real sound of each app (Windows app).</li>
      <li>Mix B (stream) records each app separately and plays it on your stream device (Windows app).</li>
      <li>The effects (EQ, compressor, gate, noise reduction) are still a preview.</li>
      <li>When you close Daelij MiX, all apps go back to 100% volume.</li>
    </ul>
  </div>
</div>

<style>
  .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(340px, 1fr)); gap: 14px; max-width: 1000px; }
  .f { display: flex; align-items: center; justify-content: space-between; gap: 12px; margin-bottom: 10px; font-size: 12px; color: var(--muted); }
  .scene { display: flex; align-items: center; justify-content: space-between; padding: 6px 0; border-top: 1px solid var(--line); }
  .steps { margin: 4px 0 8px; padding-left: 18px; color: var(--muted); font-size: 12.5px; line-height: 1.6; }
  .steps li.done { color: var(--ok); }
  .steps b { color: var(--text); }
  .small { font-size: 11px; margin: 0; }
  ul { margin: 0; padding-left: 18px; color: var(--muted); font-size: 12.5px; line-height: 1.6; }
</style>
