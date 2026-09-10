<script lang="ts">
  import "../app.css";
  import { QueryClient, QueryClientProvider } from "@tanstack/svelte-query";
  import ThemeToggle from "$lib/components/ThemeToggle.svelte";
  import { initTheme } from "$lib/theme.svelte";

  let { children } = $props();

  const queryClient = new QueryClient({
    defaultOptions: { queries: { staleTime: Infinity, retry: 1 } },
  });

  $effect(() => {
    initTheme();
  });
</script>

<svelte:head>
  <title>omapak — the open Flatpak repo</title>
  <meta
    name="description"
    content="omapak grades the artifact, not the authorship. Every submission scored on a published rubric, every judge report public."
  />
</svelte:head>

<QueryClientProvider client={queryClient}>
  <div class="flex min-h-dvh flex-col">
    <header class="border-b border-line bg-[var(--theme-header-bg)]">
      <div
        class="mx-auto flex h-14 w-full max-w-[var(--page-width)] items-center justify-between px-6"
      >
        <a href="/" class="flex items-baseline gap-2">
          <span class="font-mono text-lg font-medium text-accent">omapak</span>
          <span class="hidden font-mono text-xs text-ink-dim sm:inline"
            >grade the artifact, not the authorship</span
          >
        </a>
        <nav class="flex items-center gap-5 font-mono text-sm">
          <a href="/" class="text-muted transition-colors hover:text-fg">catalog</a>
          <a href="/rubric" class="text-muted transition-colors hover:text-fg">rubric</a>
          <a href="/submit" class="text-muted transition-colors hover:text-fg">submit</a>
          <ThemeToggle />
        </nav>
      </div>
    </header>

    <main class="mx-auto w-full max-w-[var(--page-width)] flex-1 px-6">
      {@render children()}
    </main>

    <footer class="border-t border-line">
      <div
        class="mx-auto flex w-full max-w-[var(--page-width)] flex-wrap items-center justify-between gap-3 px-6 py-5 font-mono text-xs text-ink-dim"
      >
        <span>omapak — an Outcrop Labs project · every flatpak, every distro</span>
        <span
          >we don't care who or what wrote your code — we care that the app is good</span
        >
      </div>
    </footer>
  </div>
</QueryClientProvider>
