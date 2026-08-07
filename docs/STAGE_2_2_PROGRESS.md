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
- Seed-derived island generation and landmass topology
- Deterministic downstream routing, drainage-basin IDs and flow accumulation
- River-order derivation
- Save format v4 migration in progress for derived geography
- RenderSnapshot v3 migration in progress for hydrology/landmass arrays
- Browser WASM terrain bootstrap
- Headless terrain replay reconstruction
- Three.js authoritative heightfield and sea-level mesh
- Terrain-specific source contract
- Canonical-state terrain checksum test

## Validation status

- rustfmt: pending after hydrology schema migration
- strict Clippy: pending after hydrology schema migration
- complete Rust workspace tests: pending after hydrology schema migration
- headless deterministic regression: pending after hydrology schema migration
- locked Cargo graph verification: pending
- actual browser WASM build: pending
- npm ci: pending
- TypeScript typecheck: pending
- Vite production build: pending
- Stage 2.1 regression contract: pending
- Stage 2.2 terrain/hydrology contract: pending
- final evidence and acceptance: pending
