<script lang="ts">
  import Monitor from "@lucide/svelte/icons/monitor";
  import Moon from "@lucide/svelte/icons/moon";
  import Sun from "@lucide/svelte/icons/sun";
  import { cyclePref, getPref } from "$lib/theme.svelte";

  // getPref() reads module-level $state — the template tracks it reactively.
  let label = $derived(
    getPref() === "system" ? "system" : getPref() === "mercury" ? "dark" : "light",
  );
</script>

<button
  onclick={() => cyclePref()}
  aria-label="Cycle theme (system, light, dark)"
  title="theme: {label}"
  class="flex h-8 w-8 items-center justify-center rounded border border-line-subtle text-muted transition-colors hover:border-line-strong hover:text-fg"
>
  {#if getPref() === "system"}
    <Monitor size={15} />
  {:else if getPref() === "mercury"}
    <Moon size={15} />
  {:else}
    <Sun size={15} />
  {/if}
</button>
