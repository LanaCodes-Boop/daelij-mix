<script>
  import { I } from '../lib/icons.js';
  let { name, size = 16 } = $props();
</script>
<svg class="ico" viewBox="0 0 24 24" aria-hidden="true" style="width:{size}px;height:{size}px">{@html I[name] ?? ''}</svg>
