<script lang="ts">
  import { VERDICT_LABEL, type Verdict } from "$lib/report";

  let {
    verdict,
    certified = false,
  }: { verdict: Verdict | "unpublished"; certified?: boolean } = $props();

  const style: Record<string, string> = {
    published: "border-success/40 text-success",
    build_failed: "border-danger/40 text-danger",
    unpublished: "border-line-strong text-muted",
  };
</script>

<span class="inline-flex items-center gap-1.5">
  {#if certified}
    <span
      class="inline-flex items-center gap-1 rounded-sm border border-accent-border bg-accent-soft px-2 py-0.5 font-mono text-xs text-accent"
      title="Meets omapak Certified criteria: valid appstream, advisory average ≥ 3.0, packaging ≥ 3/5, no critical security flags"
    >
      ✓ certified</span
    >
  {/if}
  <span
    class="inline-flex items-center rounded-sm border px-2 py-0.5 font-mono text-xs
      {style[verdict] ?? style.unpublished}"
  >
    {verdict === "unpublished" ? "unpublished" : VERDICT_LABEL[verdict]}
  </span>
</span>
