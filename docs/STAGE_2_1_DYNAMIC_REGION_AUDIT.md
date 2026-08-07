# Stage 2.1 Dynamic Region Audit

Status: **PASS**

Validation branch: `feature/stage-2-1-dynamic-region-audit`
Validated implementation head: `82102d8b7efd8052664a73663cb08217e0abe936`

WBS v2.0 re-review scope:

- 2.1.3 world coordinate/size validity under variable Region counts
- 2.1.4 former 60-Region identity assumptions
- 2.1.5 adjacency graph behavior under variable Region counts

## Audit findings

### 2.1.3 World coordinates and size — PASS

`WorldBounds` and `MapPoint` are expressed in integer metres and are not derived from Region count. The terrain heightfield owns its own bounds/grid dimensions. No coordinate calculation requires 60 Regions.

The existing coordinate model is retained. Variable-count topology tests certify that changing Region count does not change the coordinate contract.

### 2.1.4 Region identity — PASS

The previous validator contained fixed-count assumptions:

- `TRIAL_REGION_COUNT = 60` was treated as a core invariant
- initialized topology required exactly 60 entries
- Region ID was assumed to equal `vector_index + 1`
- lookup used `id - 1` as a direct vector index

Refactor completed:

- `TRIAL_REGION_COUNT = 60` remains only as Standard Benchmark S fixture data
- initialized topology accepts any positive Region count representable by the current collection
- Region IDs must be non-zero, unique and strictly ascending
- IDs may be sparse/non-contiguous
- lookup uses ID binary search rather than index identity
- an initialized spatial state with zero Regions remains invalid; `(None, [])` is the sole uninitialized representation

The current `RegionId(u16)` representation is intentionally not generalized into the common Stable Entity ID in this audit. Stable cross-entity identity is WBS 2.5.1 and Region Registry migration is WBS 2.5.4.

### 2.1.5 Adjacency graph — PASS

The previous validator rejected neighbor IDs above 60 even if a Region existed.

Refactor completed:

- neighbor validity is based on actual Region existence
- adjacency remains reciprocal
- neighbor lists remain strictly ascending and unique
- sparse Region IDs are supported
- no adjacency rule depends on Standard Benchmark count

## Persistence and rendering audit

The binary save codec writes a dynamic Region count before serializing Region records and allocates based on the decoded count. It does not use 60 as its binary collection length.

The RenderSnapshot protocol maps `world.spatial.regions.iter()` into a variable-length `Vec<RenderRegionSnapshot>`.

No save-format bump was required for this audit because the existing collection codec was already variable-length and the serialized Region fields did not change.

## Validation evidence

Permanent Core CI run `31203814765` — **PASS**

Verified:

- `cargo fmt --all -- --check` — PASS
- strict Clippy with `-D warnings` — PASS
- complete locked Rust workspace tests — PASS
- new 12/60/75 Region count coverage — PASS
- sparse/non-contiguous Region ID lookup — PASS
- existence-based adjacency validation — PASS
- initialized zero-Region topology rejection — PASS
- existing save invariants — PASS
- headless save/load/replay — PASS

Permanent Viewer CI run `31203814799` — **PASS**

Verified:

- locked Rust workspace — PASS
- actual WASM adapter build — PASS
- `npm ci` — PASS
- TypeScript typecheck — PASS
- Vite production build — PASS
- Viewer ownership/source contract chain — PASS
- updated Stage 2.1 source contract — PASS

## WBS restoration

- [x] 2.1.3 world coordinate/size valid under variable Region structure
- [x] 2.1.4 Region identity no longer treats 60 as a core limit
- [x] 2.1.5 adjacency graph validated for variable/sparse Region identity

All three v2.0 re-review items satisfy the completion rule again: implementation exists, integration validation passed, and evidence is recorded.

## Final decision

**Stage 2.1 dynamic Region re-review: PASS**

Next development target: Stage 2.5 `Dynamic Entity Architecture`.