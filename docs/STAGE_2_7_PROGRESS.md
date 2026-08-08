# Stage 2.7 Camera and Selection

Status: **PASS**

Authoritative scope: WBS v3.1 civilization-origin revision.

## WBS v3.1 completion

- ■ 2.7.1 world-scale camera
- ■ 2.7.2 zoom / rotate / move
- ■ 2.7.3 camera range limits
- ■ 2.7.4 HumanGroup / Settlement / Community / PoliticalEntity / Country / Region selection contract
- ■ 2.7.5 selected-target highlight
- ■ 2.7.6 selected-entity focus and world framing
- ■ 2.7.7 spatial-event automatic focus hook for war / mass migration

## Architecture fixed by Stage 2.7

- terrain, Region topology, selection and camera focus share one `WorldSpaceTransform`
- `WorldCameraController` owns Viewer camera navigation rules over Three.js `OrbitControls`
- zoom, rotate and pan are enabled with deterministic Viewer-side distance, polar-angle and target bounds
- Region and HumanGroup records can be ray-cast through transparent Viewer-only hit proxies without mutating Simulation Core state
- the selection type contract is already extensible to Settlement, Community, PoliticalEntity and Country records when later RenderSnapshot stages expose them
- click selects, double-click selects and focuses, the selection panel can select/focus, and the world control restores world framing
- selected Regions receive a separate outline; point-like selected entities receive a separate reticle
- war and mass-migration spatial events have a Viewer-side automatic focus interface without inventing the later domain event generators
- Stage 2.7 does not pre-implement Stage 2.8 final entity models or political/transport/military rendering

## Permanent validation

Clean functional/evidence candidate head before this documentation commit:

- PR #15
- Core run `31243287451`: PASS
  - rustfmt PASS
  - strict Clippy PASS
  - full Rust workspace tests PASS
  - headless trial/replay PASS
  - Stage 1 command contract PASS
- Viewer run `31243287371`: PASS
  - locked Rust workspace PASS
  - real Rust WASM build PASS
  - npm install PASS
  - TypeScript typecheck PASS
  - Vite production build PASS
  - chained Viewer contracts through Stage 2.7 PASS
  - Viewer preview artifact upload PASS
- Preview artifact ID: `9017707869`
- Preview artifact digest: `sha256:3b69de09069e7bd78314e12eaddd800838d479a0ec708e6a168b1e73f767e559`

This PASS documentation commit must itself pass the same permanent Core and Viewer gates before merge.

## Preview delivery

- PR Viewer CI retains the built `viewer/dist` directory as a downloadable artifact.
- `develop` owns a `Viewer Preview` workflow that rebuilds the same Rust WASM + TypeScript/Three.js Viewer and deploys it through GitHub Pages.
- the Pages workflow also retains `viewer/dist` as a fallback artifact.
- GitHub Pages deployment is verified after PR #15 merges to `develop`; deployment availability is a delivery check, not an excuse to weaken Stage 2.7 code validation.

## Explicit non-goals

- no final HumanGroup 3D model
- no Settlement/City/Country visual model
- no political territory shading or dynamic borders
- no roads, trade, military or battle render
- no domain war/migration generation
