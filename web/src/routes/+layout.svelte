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
  <title>Omapak · the open Flatpak repo</title>
  <meta
    name="description"
    content="Omapak grades the app on its own merits. Every submission scored by an open agent judge, every report public, never gatekept."
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
        </a>
        <nav class="flex items-center gap-5 font-mono text-sm">
          <a href="/" class="text-muted transition-colors hover:text-fg">catalog</a>
          <a href="/mission" class="text-muted transition-colors hover:text-fg">mission</a>
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
        <span>Omapak · every flatpak, every distro</span>
        <span>one remote · every app · normal people don't give a shit how it was built, neither do we</span>
      </div>
    </footer>
  </div>
</QueryClientProvider>
