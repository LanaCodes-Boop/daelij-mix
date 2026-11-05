<script>
  import { S, commit, defaultMix, save } from '../lib/state.svelte.js';
  import { isTauri } from '../lib/engine.js';
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
    <p class="sub">Daelij MiX 0.1 · {isTauri ? 'Windows app' : 'web demo'}</p>
    <ul>
      <li>Game, Chat, Media and Aux control the real volume of the apps on them (Windows app).</li>
      <li>Meters follow the real sound of each app (Windows app).</li>
      <li>Mix B (stream) and the effects are a preview for now: they need a virtual audio driver.</li>
      <li>When you close Daelij MiX, all apps go back to 100% volume.</li>
    </ul>
  </div>
</div>

<style>
  .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(340px, 1fr)); gap: 14px; max-width: 1000px; }
  .f { display: flex; align-items: center; justify-content: space-between; gap: 12px; margin-bottom: 10px; font-size: 12px; color: var(--muted); }
  .scene { display: flex; align-items: center; justify-content: space-between; padding: 6px 0; border-top: 1px solid var(--line); }
  ul { margin: 0; padding-left: 18px; color: var(--muted); font-size: 12.5px; line-height: 1.6; }
</style>
