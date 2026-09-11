import adapter from "@sveltejs/adapter-static";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    // An empty catalog (zero published apps) leaves /app/[id] uncrawled;
    // that's a legitimate state, not an error.
    prerender: { handleUnseenRoutes: "ignore" },
    adapter: adapter({
      // Fully prerendered static catalog — deployed to R2 / any static host.
      pages: "build",
      assets: "build",
      fallback: 'index.html',
      precompress: true,
      strict: false,
    }),
  },
};

export default config;
