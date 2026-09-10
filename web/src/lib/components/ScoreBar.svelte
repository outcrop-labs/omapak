<script lang="ts">
  let { score }: { score: number } = $props();
  let clamped = $derived(Math.max(0, Math.min(5, Math.round(score))));
</script>

<span class="inline-flex items-center gap-2 whitespace-nowrap">
  <span class="font-mono tracking-tight text-accent"
    >{"■".repeat(clamped)}<span class="text-ink-dim">{"□".repeat(5 - clamped)}</span></span
  >
  <span class="font-mono text-xs text-muted">{clamped}/5</span>
</span>
