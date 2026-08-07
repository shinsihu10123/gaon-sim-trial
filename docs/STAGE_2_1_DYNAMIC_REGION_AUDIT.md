# Stage 2.1 Dynamic Region Audit

Status: **UNDER VALIDATION**

WBS v2.0 re-review scope:

- 2.1.3 world coordinate/size validity under variable Region counts
- 2.1.4 former 60-Region identity assumptions
- 2.1.5 adjacency graph behavior under variable Region counts

## Audit findings

### 2.1.3 World coordinates and size

`WorldBounds` and `MapPoint` are expressed in integer metres and are not derived from Region count. The terrain heightfield owns its own bounds/grid dimensions. No coordinate calculation requires 60 Regions.

Decision before CI: retain the coordinate model and certify it with variable-count topology tests.

### 2.1.4 Region identity

The previous validator contained fixed-count assumptions:

- `TRIAL_REGION_COUNT = 60` was treated as a core invariant
- initialized topology required exactly 60 entries
- Region ID was assumed to equal `vector_index + 1`
- lookup used `id - 1` as a direct vector index

Refactor under validation:

- keep `TRIAL_REGION_COUNT = 60` only as Standard Benchmark S fixture data
- initialized topology accepts variable Region counts
- Region IDs must be non-zero, unique and strictly ascending
- IDs may be sparse/non-contiguous
- lookup uses ID binary search rather than index identity

The current `RegionId(u16)` representation is not being generalized into the common Stable Entity ID in this audit. Stable cross-entity identity is WBS 2.5.1 and Region Registry migration is WBS 2.5.4.

### 2.1.5 Adjacency graph

The previous validator rejected neighbor IDs above 60 even if a Region existed.

Refactor under validation:

- neighbor validity is based on actual Region existence
- adjacency remains reciprocal
- neighbor lists remain strictly ascending and unique
- sparse Region IDs are supported
- no adjacency rule depends on Standard Benchmark count

## Persistence and rendering audit

The current binary save codec already writes a dynamic Region count before serializing Region records and allocates based on the decoded count. It therefore does not use 60 as its binary collection length.

The RenderSnapshot protocol already maps `world.spatial.regions.iter()` into a variable-length `Vec<RenderRegionSnapshot>`.

## Completion gate

The three re-review items return to PASS only after:

1. rustfmt PASS
2. strict Clippy PASS
3. complete Rust workspace tests PASS
4. headless save/load/replay PASS
5. Viewer/WASM production path PASS
6. Stage 2.1 source contract PASS with fixed-count core assumptions explicitly forbidden
7. final evidence recorded on the validated head
