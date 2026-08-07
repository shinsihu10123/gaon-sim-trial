# Stage 2.2 Validation Record

Status: **PASS**

Validated head before evidence commit: `57389144697fb06578e2fce1e15ac96294590668`

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
- Save format v3 retained: canonical terrain is serialized; deterministic hydrology and landmass topology are regenerated rather than duplicated
- RenderSnapshot v3 terrain, hydrology and landmass arrays
- Browser WASM authoritative terrain bootstrap
- Headless terrain replay reconstruction
- Save/load/replay equality assertions for derived hydrology and landmasses
- Three.js authoritative heightfield, sea-level mesh and river network
- Stage 2.1 and Stage 2.2 source-contract gates
- Canonical-state terrain checksum test

## Core CI evidence

Permanent workflow: `Stage 1 Core CI`
Run: `31185062941`
Result: **PASS**

- `cargo fmt --all -- --check`: PASS
- `cargo clippy --locked --workspace --all-targets -- -D warnings`: PASS
- Rust tests: **69 / 69 PASS**
- headless deterministic regression: PASS
- accepted-command contract: PASS

Headless output for seed 1 after 365 simulated days:

- date: `0002-01-01`
- elapsed days: `365`
- ticks: `365`
- executed commands: `2`
- emitted events: `2`
- command journal entries: `2`
- render stride at 365×: `13`
- terrain samples: `16,641`
- land samples: `7,371`
- river samples: `2,145`
- drainage basins: `409`
- island landmasses: `1`
- save metadata bytes: `252`
- save state bytes: `100,100`
- canonical authoritative digest: `44411b984731dcaa`
- deterministic random probe: `821968d634b31c8f`

The headless regression also confirms that save/load and command-journal replay regenerate identical hydrology and landmass topology from the canonical heightfield.

## Viewer CI evidence

Permanent workflow: `Stage 1 Viewer CI`
Run: `31185062972`
Result: **PASS**

- locked Rust workspace check: PASS
- actual `wasm32-unknown-unknown` adapter build: PASS
- `wasm-pack 0.13.1` release package: PASS
- `npm ci`: PASS
- TypeScript `tsc --noEmit`: PASS
- Vite production build: PASS
- Stage 1.1 browser ownership contract: PASS
- Stage 2.1 world-data contract: PASS on save format v3
- Stage 2.2 terrain/hydrology contract: PASS on save v3 / RenderSnapshot v3

Production build observations:

- generated WASM: approximately `210.10 kB` (`92.23 kB` gzip)
- main JavaScript chunk: approximately `539.79 kB` (`136.05 kB` gzip)
- worker JavaScript: approximately `4.30 kB`

The >500 kB JavaScript chunk warning is non-blocking for Stage 2.2 and should be revisited during later UI/performance optimization.

## WBS acceptance

All Stage 2.2 items are satisfied:

- [x] 2.2.1 continuous terrain mesh
- [x] 2.2.2 heightmap generation
- [x] 2.2.3 coastline generation
- [x] 2.2.4 plains / hills / mountains
- [x] 2.2.5 forest / desert / wetland distribution
- [x] 2.2.6 river basins
- [x] 2.2.7 islands
- [x] 2.2.8 ocean surface
- [x] 2.2.9 natural terrain transition / slope control
- [x] 2.2.10 terrain-seed reproducibility

## Acceptance decision

**Stage 2.2: PASS**

The evidence commit containing this record must itself pass both permanent Core and Viewer CI before PR #9 is merged into `develop`.
