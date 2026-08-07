# Stage 2.3 Validation Record

Status: **IN PROGRESS**

## Implemented

- deterministic `TerrainEffects` fixed-point schema
- agriculture yield multiplier
- construction cost multiplier
- movement cost multiplier
- defense multiplier
- port feasibility
- natural carrying capacity in people/km²
- geography-only productivity multiplier
- hydrology-aware river-mouth and river-order effects
- region-ready sample-index aggregation
- generated-world deterministic/range tests
- permanent Stage 2.3 source-contract validation chained into Viewer CI
- calibration assumptions documented separately from empirical claims

## Pending validation

- rustfmt
- strict Clippy
- complete locked Rust workspace tests
- existing Stage 1/2 regression suite
- actual WASM build
- npm ci
- TypeScript typecheck
- Vite production build
- Stage 2.1 / 2.2 / 2.3 source contracts
- final PASS evidence
