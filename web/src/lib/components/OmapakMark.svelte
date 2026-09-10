<script lang="ts">
  /**
   * The omapak mark: one gold chevron over its reflection gap, the "A" of
   * the name reduced to two strokes. Companion to the favicon (same
   * geometry); accent + currentColor only, so both Mercury modes stay
   * correct without hardcoding. Dead simple on purpose.
   */
  let { class: className, size }: { class?: string; size?: number } = $props();
</script>

<svg
  viewBox="0 0 32 32"
  class={className}
  style:width={size ? `${size}px` : undefined}
  style:height={size ? `${size}px` : undefined}
  aria-hidden="true"
>
  <path
    d="M8 22V10l8 8 8-8v12"
    fill="none"
    stroke="var(--theme-accent)"
    stroke-width="3"
    stroke-linecap="round"
    stroke-linejoin="round"
  />
</svg>
