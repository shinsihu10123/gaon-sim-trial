# Stage 1 Foundation — Final Acceptance Record

## Decision

**Status: PASS**

Stage 1 Foundation is complete. Every Stage 1 subsystem has implementation, automated validation and recorded evidence.

## Accepted scope

### 1.1 Development environment and Viewer foundation — PASS

- GitHub repository and branch strategy
- Rust workspace
- TypeScript/Vite development environment
- Three.js viewport foundation
- renderer-facing `RenderSnapshot` protocol
- Rust `simulation-wasm` adapter
- Web Worker ownership of the browser simulation engine
- typed main-thread bridge
- locked Rust and Viewer dependency graphs
- actual browser WASM generation
- TypeScript typecheck and Vite production build

Evidence: `docs/STAGE_1_1_EVIDENCE.md`

### 1.2 Simulation time and fixed daily tick — PASS

- Year 1 / Month 1 / Day 1 start
- deterministic 365-day calendar
- 1 tick = 1 simulated day
- month/quarter/year boundaries
- pause
- 1x / 7x / 30x / 90x / 365x playback
- one-day and 30-day manual stepping
- integer nanosecond accumulation
- simulation/render timing separation

Evidence: `docs/STAGE_1_2_EVIDENCE.md`

### 1.3 Command processing — PASS

- user and country-AI command sources
- immediate and scheduled commands
- deterministic command IDs
- deterministic priority lanes
- canonical ordering `(execution date, priority, command id)`
- command validation
- pending queue and execution history

Evidence: `docs/STAGE_1_3_EVIDENCE.md`

### 1.4 Event Ledger — PASS

- append-only immutable event history
- monotonic Event IDs
- event date and tick position
- event category taxonomy
- user/country/system source attribution
- command execution facts
- category/source/date filtering
- identical event order under manual and high-speed playback

Evidence: `docs/STAGE_1_4_EVIDENCE.md`

### 1.5 Save and restore — PASS

- versioned `metadata.json` + canonical `state.bin`
- manual and autosave snapshots
- complete Stage 1 authoritative engine snapshot
- checksum corruption detection
- metadata/binary consistency validation
- transactional restore
- snapshot invariant validation
- future pending-command preservation
- 50-year save → restore → additional 50-year exact continuation test

Evidence: `docs/STAGE_1_5_EVIDENCE.md`

### 1.6 Determinism, digest and replay — PASS

- canonical authoritative state digest
- pending/executed commands and Event Ledger included in digest
- counter-based deterministic random contract
- accepted-command journal
- fresh-engine command replay
- exact 10,000-tick replay
- manual/365x authoritative equality
- committed Cargo lockfile
- Cargo `--locked` validation

Evidence: `docs/STAGE_1_6_EVIDENCE.md`

## Single-source-of-truth architecture

The accepted Stage 1 ownership rule is:

```text
SimulationEngine owns authoritative state
UI/AI submit Commands
SimulationEngine produces Events
simulation-save persists canonical authoritative state
simulation-protocol projects immutable RenderSnapshot data
simulation-wasm hosts the Rust engine in the browser
Web Worker owns the browser WASM engine instance
Three.js reads RenderSnapshot data only
```

No presentation component owns authoritative simulation state.

## Deterministic baseline

The accepted headless regression baseline remains:

```text
date=0002-01-01 elapsed_days=365 ticks=365 commands=2 events=2 journal=2 render_stride=13 save_json_bytes=249 save_state_bytes=218 canonical_digest=5c4df237597b9f79 random_probe=6f6ec8e35d96b3d5
```

Stage 1.1 additionally proves this core can be packaged into browser-compatible WebAssembly and consumed through the Worker/Viewer boundary without transferring state ownership.

## Automated validation baseline

At Stage 1 completion the Rust workspace contains the previously accepted 39 Stage 1.2–1.6 unit tests plus 4 protocol/WASM tests, for an expected total of **43 Rust unit tests**.

The full-stack Stage 1.1 certification additionally passes:

- Rust format
- strict Clippy
- locked Rust tests
- headless runner
- wasm32 build
- wasm-bindgen browser bindings
- locked Viewer install through `npm ci`
- TypeScript typecheck
- Vite production build
- Viewer ownership/source contract

## Explicit non-scope

Stage 1 does not claim completion of:

- terrain or elevation
- 60-region world topology
- country borders
- cities/resources/infrastructure
- economic production or prices
- population
- trade
- diplomacy
- military or war
- Stage 2 spatial overlays

The neutral Three.js staging surface is infrastructure only and is not presented as simulated geography.

## Stage 2 entry conditions

Stage 2 may now begin because:

1. authoritative state ownership is fixed;
2. daily time progression is deterministic;
3. future UI/AI actions have a command boundary;
4. resulting facts have an Event Ledger boundary;
5. authoritative state can be saved and restored exactly;
6. state can be hashed and replayed deterministically;
7. the browser renderer has a Worker/WASM/RenderSnapshot boundary that can be extended without rewriting the core ownership model.

## Final Stage 1 judgment

**PASS — Stage 2 world and 3D foundation development is authorized.**
