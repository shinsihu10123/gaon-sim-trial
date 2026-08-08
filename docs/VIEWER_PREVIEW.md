# Viewer Preview

The latest development Viewer is automatically built and deployed from the `develop` branch by `.github/workflows/viewer-preview.yml`.

## Live preview

https://shinsihu10123.github.io/gaon-sim-trial/

## Deployment contract

- source branch: `develop`
- runtime: Rust WASM + Web Worker + TypeScript + Three.js
- build: Vite production build
- Pages base path: `/gaon-sim-trial/`
- deployment trigger: every push to `develop` or manual workflow dispatch
- CI deployment pipeline: Viewer build -> contract verification -> Pages artifact -> GitHub Pages deployment

The preview is a development inspection surface, not a release build. Features appear here only after they are merged into `develop` and pass the Viewer build pipeline.
