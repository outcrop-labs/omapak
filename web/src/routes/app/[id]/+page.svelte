<script lang="ts">
  import { page } from "$app/state";
  import ReportView from "./ReportView.svelte";

  const id = $derived(page.params.id ?? "");
</script>

{#key id}
  <ReportView appId={id} />
{/key}
