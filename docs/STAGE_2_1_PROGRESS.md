# Stage 2.1 Validation Record

Status: **PASS**

## Accepted implementation

- Authoritative `WorldSpatialState`: PASS
- Fixed TEST world region count (`60`): PASS
- Canonical RegionId order (`1..=60`): PASS
- Integer-metre world coordinates: PASS
- Continuous `WorldBounds`: PASS
- Land/Ocean region classification: PASS
- Polygon boundary validation: PASS
- Reciprocal, sorted adjacency graph: PASS
- Separate `legal_owner` / `controller`: PASS
- Canonical uninitialized Stage 1 world representation: PASS
- Save format v2 spatial serialization: PASS
- Spatial topology included in canonical state binary/checksum: PASS
- RenderSnapshot bounds/region topology contract: PASS
- Stage 1.1 browser source repair: PASS
- Rust `simulation-protocol` and `simulation-wasm` source trees: PASS
- Web Worker ownership of authoritative Rust engine: PASS
- TypeScript/Vite/Three.js Viewer source tree: PASS
- Cargo/npm lockfiles committed from validated resolver output: PASS

## Permanent validation gates

### Stage 1 Core CI

- Run: `31166822474`
- rustfmt: PASS
- strict Clippy (`--locked`, `-D warnings`): PASS
- command/time/event/save/determinism contract chain: PASS
- Rust non-doc tests: **53/53 PASS**
- headless regression: PASS

### Stage 1 Viewer CI

- Run: `31166822466`
- locked Rust workspace check: PASS
- real `wasm32-unknown-unknown` / wasm-pack build: PASS
- `npm ci` from committed package lock: PASS
- TypeScript typecheck: PASS
- Vite production build: PASS
- Stage 1.1 browser ownership contract: PASS
- Stage 2.1 world data contract: PASS

## Lockfile provenance

- Bootstrap resolver/persistence run: `31166436665` — PASS
- `Cargo.lock`: committed and validated
- `viewer/package-lock.json`: committed and validated
- Temporary Stage 2.1 bootstrap workflow removed after dependency graphs were fixed.

## Determinism evidence

Headless output from the permanent Core CI:

```text
date=0002-01-01 elapsed_days=365 ticks=365 commands=2 events=2 journal=2 render_stride=13 save_json_bytes=249 save_state_bytes=219 canonical_digest=38d9b7aac32f6f99 random_probe=821968d634b31c8f
```

- Canonical digest: `38d9b7aac32f6f99`
- Deterministic random probe: `821968d634b31c8f`

## Stage boundary

Stage 2.1 defines identity, topology, ownership/control and persistence/render contracts only. Detailed elevation, terrain generation, hydrology, biome assignment, resources, cities and country placement remain later Stage 2 work.

**Final Stage 2.1 decision: PASS.**
