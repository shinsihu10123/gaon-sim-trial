# Stage 2.2 Validation Record

Status: **IN PROGRESS**

## Implemented

- Canonical 129×129 terrain state
- Integer-metre elevation and moisture substrate
- Deterministic `simulation-worldgen` crate
- Multi-scale value-noise continental generation
- Ridge elevation contribution
- Deterministic sea-threshold calibration
- Forced continuous outer ocean
- Relief and biome classification
- Deterministic adjacent-slope limiter
- Seed-derived island generation
- Deterministic connected-landmass and island IDs derived from the canonical heightfield
- Deterministic downstream routing, drainage-basin IDs and flow accumulation derived from the canonical heightfield
- River-order derivation
- Save format v3 retained: the canonical heightfield is serialized and derived geography is regenerated, avoiding duplicated state
- RenderSnapshot v3 hydrology and landmass arrays
- Browser WASM terrain bootstrap
- Headless terrain replay reconstruction
- Save/load/replay equality assertions for derived hydrology and landmasses
- Three.js authoritative heightfield, sea-level mesh and river network
- Terrain-specific source contract
- Canonical-state terrain checksum test

## Validation status

- rustfmt: normalization triggered
- strict Clippy: pending after formatting
- complete Rust workspace tests: pending after formatting
- headless deterministic regression: pending after formatting
- locked Cargo graph verification: pending
- actual browser WASM build: pending
- npm ci: pending
- TypeScript typecheck: pending
- Vite production build: pending
- Stage 2.1 regression contract: pending
- Stage 2.2 terrain/hydrology contract: pending
- final evidence and acceptance: pending
