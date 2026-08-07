# Stage 1.1 Evidence Record

## Scope

Stage 1.1 establishes the repository, Rust workspace, browser development environment, typed renderer protocol, WebAssembly adapter, Web Worker ownership boundary, and Three.js/Vite viewer shell required by every later stage.

The validated runtime boundary is:

```text
Rust SimulationEngine
→ simulation-protocol RenderSnapshot
→ simulation-wasm / wasm-bindgen
→ Web Worker
→ typed TypeScript SimulationBridge
→ Three.js renderer
```

The simulation core remains the single source of truth. Browser UI state, camera state and render timing are not authoritative simulation state.

## WBS coverage

- `1.1.1` GitHub repository and branch strategy — COMPLETE
- `1.1.2` Rust simulation-core workspace — COMPLETE
- `1.1.3` TypeScript UI foundation — COMPLETE
- `1.1.4` Vite development/build environment — COMPLETE
- `1.1.5` Three.js renderer foundation — COMPLETE
- `1.1.6` core/viewer bridge contract — COMPLETE
- `1.1.7` browser WebAssembly connection — COMPLETE
- `1.1.8` Web Worker isolation/ownership — COMPLETE
- `1.1.9` documented build/run commands — COMPLETE
- `1.1.10` automated CI validation — COMPLETE

## Rust workspace

The Stage 1 workspace now contains:

- `simulation-model`
- `simulation-core`
- `simulation-save`
- `simulation-runner`
- `simulation-protocol`
- `simulation-wasm`

`simulation-protocol` owns the renderer-facing DTO boundary rather than exposing mutable core internals.

`RenderSnapshot` protocol v1 currently contains only real Stage 1 data:

- simulation date
- elapsed simulated days
- pending command count
- executed command count
- Event Ledger count
- canonical authoritative digest

No fictitious terrain, countries, cities, industries or economic statistics are inserted before their corresponding domain stages.

## WebAssembly adapter

`simulation-wasm` is compiled as `cdylib`/`rlib` using `wasm-bindgen`.

The adapter provides:

- exact 64-bit seed initialization via fixed-width hexadecimal text
- immutable snapshot serialization
- deterministic explicit daily stepping
- protocol version

The hexadecimal seed boundary prevents JavaScript number precision loss.

## Web Worker ownership

`simulation.worker.ts` loads the generated WASM package and owns the `SimulationWasm` instance.

The main thread never owns `SimulationEngine` or `SimulationWasm`. It communicates through typed request/response messages using `WorkerSimulationBridge`.

This prevents renderer frame timing, DOM interactions and Three.js camera operations from mutating authoritative simulation state.

## Three.js viewer foundation

The production-buildable Stage 1 viewport includes:

- `WebGLRenderer`
- perspective camera
- `OrbitControls`
- lighting
- responsive resizing
- independent animation loop
- resource disposal
- neutral grayscale staging surface

The staging surface is explicitly not Stage 2 terrain. Actual terrain elevation, 60 regions, borders, cities and spatial world layers remain assigned to Stage 2.

The UI displays only values received from the Rust `RenderSnapshot`.

## Frontend dependency baseline

Pinned direct dependencies:

- Vite `7.1.0`
- TypeScript `5.9.2`
- Three.js `0.185.1`
- `@types/three` `0.185.1`

The Viewer dependency graph is committed in `viewer/package-lock.json` and validated with `npm ci`.

The Rust dependency graph is committed in the workspace `Cargo.lock` and validated with Cargo `--locked`.

## Full-stack certification

A success-only Stage 1.1 certification workflow was executed on the feature branch before merge. The PASS marker was created only after every command below completed successfully:

1. `cargo fmt --all -- --check`
2. `cargo clippy --locked --workspace --all-targets -- -D warnings`
3. `cargo test --locked --workspace`
4. `cargo run --locked -p simulation-runner --quiet`
5. install pinned `wasm-pack 0.13.1`
6. compile `simulation-wasm` for `wasm32-unknown-unknown`
7. generate wasm-bindgen browser JavaScript/TypeScript bindings
8. `npm ci` for the Viewer
9. TypeScript type checking
10. Vite production build
11. Stage 1.1 ownership/source contract verification

The feature was merged through pull request #6 only after the final certification marker existed.

## Rust regression coverage

The workspace contains the previously validated Stage 1.2–1.6 tests plus the new protocol/WASM tests.

Expected workspace unit-test composition at the certified revision:

- `simulation-core`: 26
- `simulation-model`: 7
- `simulation-save`: 6
- `simulation-protocol`: 2
- `simulation-wasm`: 2
- total: **43 Rust unit tests**

The headless runner remains compatible with the browser additions because the Viewer and renderer do not alter authoritative state ownership.

## Existing deterministic headless baseline

The Stage 1 headless regression remains:

```text
date=0002-01-01 elapsed_days=365 ticks=365 commands=2 events=2 journal=2 render_stride=13 save_json_bytes=249 save_state_bytes=218 canonical_digest=5c4df237597b9f79 random_probe=6f6ec8e35d96b3d5
```

Stage 1.1 adds a browser projection of this core; it does not replace or weaken the headless validation path.

## Decision

Status: **PASS**

Stage 1.1 satisfies implementation + dependency locking + Rust validation + actual browser WASM generation + TypeScript validation + production Viewer build + recorded evidence.

Stage 2 may now extend `RenderSnapshot` with real world geometry and spatial state without moving authoritative ownership out of the Rust simulation core.
