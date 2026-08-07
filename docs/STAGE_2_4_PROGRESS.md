# Stage 2.4 Validation Record

Status: **PASS**

Validated feature head before evidence update: `f81bc09b7da1c154ab634d92b880567f6f029e92`
PR: #11 `Stage 2.4 deterministic resource generation`

## Implemented scope

- renewable food production base per canonical terrain cell
- deterministic finite energy resource deposits
- deterministic finite metal resource deposits
- deterministic finite timber/construction resource deposits
- separate initial and remaining reserve quantities
- resource quality in permille
- resource accessibility in permille
- bounded extraction that cannot exceed remaining stock
- depletion-ratio calculation
- canonical `ResourceCellState` aligned 1:1 to terrain samples
- `ResourceFieldState` with deterministic region-ready aggregation
- reserve-weighted aggregate quality and accessibility
- ocean-resource exclusion validation
- representative-seed scarcity/concentration balance validation
- authoritative `WorldState.resources`
- save format v4 with exact remaining-stock serialization
- canonical binary/digest sensitivity to extraction
- deterministic same-seed resource regeneration for headless replay
- identical terrain/resource initialization path for WASM and headless execution
- Stage 2.4 source contract chained into permanent Viewer CI

## Generation principle

Resource roles are not assigned to countries in advance. Resource deposits are generated before country placement from the world seed and geographic inputs. Countries will inherit resource access only from the territory they later control.

The generator combines terrain suitability with independent deterministic geological/resource signals. Terrain affects probabilities and accessibility, but terrain class alone does not force a deposit to exist.

## Persistence and determinism

Resources are authoritative simulation state because finite deposits can be depleted.

The save pipeline stores:

- initial reserve quantity
- remaining reserve quantity
- quality
- accessibility
- renewable food-production base

Extraction changes the canonical state binary and therefore changes the authoritative digest. Save/load restores the depleted remaining stock exactly.

Command-journal replay regenerates both terrain and resources from the same seed before comparing authoritative state, preventing static world-generation data from disappearing during replay.

## Validation evidence

### Permanent Core CI

Workflow run: `31192650029` — **PASS**

Verified:

- `cargo fmt --all -- --check` — PASS
- `cargo clippy --locked --workspace --all-targets -- -D warnings` — PASS
- `cargo test --locked --workspace` — PASS
- headless save/load/replay — PASS

Rust test total: **85 passed, 0 failed**

Relevant resource tests include:

- extraction cannot exceed remaining stock
- invalid resource quality is rejected
- ocean cells contain no resource deposits
- same seed and terrain reproduce the identical resource field
- changing the seed changes resource distribution
- representative seeds pass the resource-balance envelope
- resource report land-cell count matches terrain land cells
- extracted stock round-trips exactly through save/load
- extracted stock changes canonical binary/checksum
- resources without terrain are rejected

Headless reference result after 365 ticks:

- terrain samples: `16641`
- land samples: `7371`
- river samples: `2145`
- drainage basins: `409`
- islands: `1`
- save metadata bytes: `253`
- save state bytes: `1165133`
- canonical digest: `d854265b9f358cdd`
- deterministic random probe: `821968d634b31c8f`

### Permanent Viewer CI

Workflow run: `31192650994` — **PASS**

Verified:

- locked Rust workspace — PASS
- actual Rust WASM adapter build — PASS
- `npm ci` — PASS
- TypeScript typecheck — PASS
- Vite production build — PASS
- Stage 1.1 ownership contract — PASS
- chained Stage 2 source contracts, including Stage 2.4 — PASS

## WBS completion

- [x] 2.4.1 food production base placement
- [x] 2.4.2 energy resource placement
- [x] 2.4.3 metal resource placement
- [x] 2.4.4 timber/construction resource placement
- [x] 2.4.5 reserve quantity and quality schema
- [x] 2.4.6 extractable quantity and depletion structure
- [x] 2.4.7 resource-distribution balance validation

All Stage 2.4 items satisfy the project completion rule: implementation exists, unit/integration validation passed, and evidence is recorded.

## Final decision

**Stage 2.4 Resource Generation: PASS**

Next WBS item is Stage 2.5 `10-country initial placement`. No Stage 2.5 implementation is included in this Stage 2.4 PR.
