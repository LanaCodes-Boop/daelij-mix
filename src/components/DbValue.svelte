<script>
  import { fmtDb, parseDb } from '../lib/state.svelte.js';
  let { value, onset, label = 'Value' } = $props();
  let editing = $state(false);
  let text = $state('');
  function start() { text = value <= -60 ? '-inf' : value.toFixed(1); editing = true; }
  function done(ok) {
    if (!editing) return;
    editing = false;
    if (ok) { const v = parseDb(text); if (v !== null) onset(v); }
  }
  const focus = (n) => { n.focus(); n.select(); };
</script>

{#if editing}
  <input class="dbin" bind:value={text} use:focus aria-label="{label} in dB"
    onkeydown={(e) => { e.stopPropagation(); if (e.key === 'Enter') done(true); if (e.key === 'Escape') done(false); }}
    onblur={() => done(true)} />
{:else}
  <button class="dbv" ondblclick={start} onkeydown={(e) => { if (e.key === 'Enter') start(); }}
    title="Double-click to type a value" aria-label="{label}: {fmtDb(value)} dB. Press Enter to type a value.">
    {fmtDb(value)}<small>dB</small>
  </button>
{/if}
