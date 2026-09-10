<script lang="ts">
  import { VERDICT_LABEL, type Verdict } from "$lib/report";

  let { verdict }: { verdict: Verdict | "unpublished" } = $props();

  const style: Record<string, string> = {
    accept_recommended: "border-success/40 text-success",
    needs_human: "border-warning/40 text-warning",
    reject_recommended: "border-danger/40 text-danger",
    unpublished: "border-line-strong text-muted",
  };
</script>

<span
  class="inline-flex items-center rounded-sm border px-2 py-0.5 font-mono text-xs
    {style[verdict] ?? style.unpublished}"
>
  {verdict === "unpublished" ? "unpublished" : VERDICT_LABEL[verdict]}
</span>
