import { defineConfig } from "vite";

export default defineConfig({
  // GitHub Pages serves this repository below /gaon-sim-trial/.
  // Local Vite development continues to use the root path.
  base: process.env.GITHUB_ACTIONS === "true" ? "/gaon-sim-trial/" : "/",
});
