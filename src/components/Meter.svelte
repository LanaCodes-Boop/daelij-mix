<script>
  import { registerMeter } from '../lib/engine.js';
  let { key, color = '#8b9cff', label = '', horizontal = false, channels = 2 } = $props();
  let el;
  $effect(() => registerMeter(key, el, horizontal));
</script>

<div class="meter" class:h={horizontal} bind:this={el} style="--c:{color}"
  role="meter" aria-label="{label} level" aria-valuemin="-60" aria-valuemax="12" aria-valuenow="-60">
  {#each Array(channels) as _}
    <div class="bar"><div class="cover"></div><div class="peak"></div></div>
  {/each}
</div>
