<script lang="ts">
  import type { AtlasData } from '$lib/atlas';
  import { getFrameStyle } from '$lib/atlas';

  export let atlas: AtlasData;
  export let name: string;
  export let scale: number = 1;
</script>

<span
  class="pixel-sprite"
  style={getFrameStyle(atlas, name, scale)}
  aria-hidden="true"
></span>

<style>
  .pixel-sprite {
    image-rendering: pixelated;
    image-rendering: crisp-edges;
    vertical-align: middle;
  }
</style>
