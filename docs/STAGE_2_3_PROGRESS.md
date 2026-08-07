# Stage 2.3 Validation Record

Status: **PASS**

Validated functional head before evidence commit: `97dc9c36eebdd2c980b90c666b0c4ac1820ff6c4`

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
- generated-world deterministic/range tests across representative seeds
- permanent Stage 2.3 source-contract validation chained into Viewer CI
- calibration assumptions documented separately from empirical claims

## Numeric contract

- land multipliers use integer permille fixed point (`1000 = 1.00×`)
- natural carrying capacity uses people/km²
- ocean samples expose zero land effects
- effects are deterministic derivatives of canonical terrain and hydrology rather than duplicated authoritative state
- later technology, infrastructure, institutions and policy remain separate modifiers

## Core CI evidence

Permanent workflow: `Stage 1 Core CI`
Run: `31188131802`
Result: **PASS**

- `cargo fmt --all -- --check`: PASS
- `cargo clippy --locked --workspace --all-targets -- -D warnings`: PASS
- complete Rust workspace tests, including Stage 2.3 generated-world tests: PASS
- Stage 1/2 regression suite: PASS
- headless save/load/replay regression: PASS
- accepted-command contract: PASS

The Stage 2.3 tests verify:

- same-seed effect-field determinism across representative seeds
- one-to-one alignment with terrain samples
- zero land effects for ocean samples
- nontrivial agriculture/construction/movement/defense/carrying/productivity ranges
- nonzero geographic port candidates
- mountain accessibility penalty with higher defensive advantage than plains
- deterministic region-ready aggregation

## Viewer CI evidence

Permanent workflow: `Stage 1 Viewer CI`
Run: `31188132332`
Result: **PASS**

- locked Rust workspace verification: PASS
- actual WASM adapter build: PASS
- `npm ci`: PASS
- TypeScript typecheck: PASS
- Vite production build: PASS
- Stage 1.1 ownership contract: PASS
- Stage 2.1 world-data contract: PASS
- Stage 2.2 terrain/hydrology contract: PASS
- Stage 2.3 terrain simulation-effect contract: PASS

## WBS acceptance

All Stage 2.3 items are satisfied:

- [x] 2.3.1 agriculture suitability calculation
- [x] 2.3.2 construction difficulty calculation
- [x] 2.3.3 movement resistance calculation
- [x] 2.3.4 defense modifier calculation
- [x] 2.3.5 port construction feasibility calculation
- [x] 2.3.6 population carrying-capacity calculation
- [x] 2.3.7 regional productivity correction

## Acceptance decision

**Stage 2.3: PASS**

The evidence commit containing this record must itself pass both permanent Core and Viewer CI before PR #10 is merged into `develop`.
