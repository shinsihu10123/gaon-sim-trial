# Stage 2.2 — Deterministic Terrain, Hydrology and Islands

## Purpose

Stage 2.2 gives the TEST world an authoritative continuous terrain surface generated from the world seed and rendered as actual 3D geometry.

Terrain elevation is authoritative simulation state, not decorative renderer data. Hydrology and landmass topology are deterministic derivatives of that authoritative heightfield so they cannot diverge from the saved geography.

## Canonical terrain geometry

The TEST terrain uses a fixed heightfield:

- grid: **129 × 129** samples
- sample count: **16,641**
- horizontal spacing: **10,000 m**
- world extent: **-640,000 m .. +640,000 m** on both axes
- total width/depth: approximately **1,280 km × 1,280 km**
- sea level: **0 m**

The odd 129-sample side provides 128 cells per axis and a stable center sample.

## Deterministic representation

Authoritative state uses integer values:

- coordinates: integer metres
- elevation: signed 16-bit metres
- moisture: integer permille (`0..=1000`)
- relief and biome: fixed enums

Floating point is allowed only after a `RenderSnapshot` reaches Three.js. Renderer scaling, camera position and vertical exaggeration never enter authoritative state.

## World-generation architecture

Terrain generation lives in the pure `simulation-worldgen` crate rather than the daily simulation kernel.

```text
world seed
   ↓
simulation-worldgen
   ↓
canonical TerrainState heightfield
   ├─ derive_hydrology()
   └─ derive_landmasses()
   ↓
WorldState
   ↓
canonical save / digest
   ↓
RenderSnapshot v3
   ↓
Three.js terrain + sea + river network
```

Hydrology and landmass IDs are not duplicated in the save file because they are pure deterministic functions of the serialized elevation field. This removes the possibility of a saved river graph disagreeing with the heightfield that generated it.

## Height generation

The generator uses stable integer hash/value-noise operations only.

Continental elevation combines four deterministic scales:

- scale 64 — broad continental mass
- scale 32 — secondary landform
- scale 16 — regional structure
- scale 8 — local detail

The scales are combined with weights `8 : 4 : 2 : 1`.

A separate ridge signal increases mountain elevation on sufficiently continental land.

### Sea-level calibration

Raw continental values are sorted deterministically and the 55th-percentile signal is used as the zero-elevation reference. The target is therefore approximately 45% land samples before edge forcing.

This prevents individual seeds from producing almost-all-land or almost-all-ocean worlds while preserving different continent shapes.

### Outer ocean

The outer three sample rings are forced below sea level. This guarantees that the world is surrounded by continuous ocean and later maritime routing does not depend on accidental map-edge land bridges.

### Natural transition / slope limit

After height generation, adjacent 10 km samples are processed by a deterministic slope limiter. The Stage 2.2 validation bound is **3,000 m maximum elevation difference between orthogonally adjacent samples**.

The limiter removes accidental near-vertical discontinuities without flattening the mountain system or introducing floating-point smoothing into authoritative state.

## Moisture and biome substrate

Moisture uses a separate deterministic noise field with broad and regional scales. Values are normalized to integer permille.

Stage 2.2 derives these compact biome classes:

- `Ocean`
- `Grassland`
- `Forest`
- `Desert`
- `Wetland`
- `Alpine`

This provides the required forest/desert/wetland distribution for the TEST world. It is intentionally not a full climate circulation or Köppen model.

## Relief classification

Relief is derived from elevation and immediate coastal adjacency:

- `DeepOcean`: below -1,200 m
- `ShallowOcean`: -1,200 m to below sea level
- `Coast`: land sample adjacent to an ocean sample
- `Plains`: inland land below 450 m
- `Hills`: 450 m to below 1,500 m
- `Mountains`: 1,500 m and above

Relief and biome remain separate concepts so later economic/military effects can use geometry independently from ecological cover.

## Island generation and landmass topology

The TEST world must contain islands rather than depending on chance. `simulation-worldgen` therefore uses the world seed to search deterministically for a sufficiently isolated ocean patch and raises a compact island inside it.

The island location is not fixed across seeds. The generator changes the canonical elevation field, so the island is automatically included in save v3, authoritative digest, replay and 3D terrain.

Connected-land analysis then assigns deterministic `landmass_id` values using 8-neighbor connectivity:

- ocean samples: landmass ID `0`
- land components: positive deterministic IDs
- largest component: mainland
- every other disconnected component: island landmass

The landmass analysis is derived from elevation and is therefore regenerated rather than independently serialized.

## Drainage basins and river generation

`TerrainState::derive_hydrology()` builds a deterministic D8-style drainage graph from the heightfield.

For every land sample:

1. inspect the eight adjacent samples;
2. select the strictly lower neighbor with the lowest elevation;
3. break equal-elevation choices by stable sample index;
4. if no lower neighbor exists, treat the sample as an inland sink;
5. accumulate upstream contributing cells from high elevation to low elevation;
6. assign a deterministic drainage-basin ID by ocean outlet or inland sink;
7. derive river order from accumulated flow.

River-order thresholds for the TEST substrate are:

- order 0: not rendered as a river
- order 1: flow accumulation ≥ 8
- order 2: flow accumulation ≥ 32
- order 3: flow accumulation ≥ 128

This is a strategic-scale hydrology model appropriate for a 10 km grid. Detailed erosion, meanders, tributary channel geometry and seasonal discharge are outside Stage 2.2.

## Persistence and authoritative digest

Save format remains **v3**.

The canonical binary stores:

- terrain bounds
- width and height
- spacing
- sea level
- every sample elevation
- every sample moisture value
- relief code
- biome code

Hydrology, basin IDs, river order and landmass IDs are not duplicated in the save because they are reproducible derivatives of these canonical terrain samples.

`authoritative_state_digest()` hashes the canonical binary substrate. Changing a valid terrain elevation changes both the saved state and every derived geography result that depends on it.

## Replay semantics

Stage 1 command-journal replay reconstructs dynamic command/event history from a seed and accepted command journal.

From Stage 2 onward, deterministic static world-generation state must also be reconstructed from the same seed. The Stage 2.2 headless path therefore:

1. replays the command journal;
2. regenerates terrain from the original seed;
3. inserts it into the replay snapshot;
4. validates the snapshot through save v3;
5. recomputes hydrology and landmass topology;
6. compares complete authoritative state, derived geography and digest.

This prevents generated geography from becoming an untracked external dependency.

## RenderSnapshot v3

Renderer protocol version advances to **v3** and includes:

- terrain dimensions
- spacing and sea level
- elevation array
- moisture array
- relief-code array
- biome-code array
- downstream sample indices
- drainage-basin IDs
- flow accumulation
- river order
- landmass IDs
- island landmass ID list

The derived arrays are calculated from the authoritative heightfield immediately before snapshot construction. TypeScript never generates an independent terrain or river model.

## Three.js representation

The Viewer builds an indexed `BufferGeometry` directly from authoritative elevation samples.

- 16,641 terrain vertices
- two triangles per heightfield cell
- computed vertex normals
- biome/relief vertex colors
- separate sea-level plane
- presentation-only vertical exaggeration
- river `LineSegments` following authoritative downstream indices

The terrain, water, river and region-topology layers remain separate scene objects. Later borders, cities, roads, resources and armies can therefore be overlaid without replacing terrain state.

## Deferred scope

Stage 2.2 does not implement:

- erosion simulation
- detailed climate circulation and seasonal weather
- fine channel geometry below the 10 km sample scale
- terrain economic modifiers — Stage 2.3
- resource deposits — Stage 2.4
- final 60-region placement on generated geography — later Stage 2 work
- country borders or cities
- selectable region interaction

## Acceptance criteria

Stage 2.2 is PASS only when all of the following hold:

1. 129×129 canonical terrain validates;
2. same seed generates identical terrain;
3. different seeds produce different terrain;
4. representative seeds maintain balanced land/ocean distribution;
5. all outer-edge samples are ocean;
6. deep ocean, shallow ocean, coast, plains, hills and mountains are generated;
7. forest, desert, wetland and other required biome classes are generated;
8. adjacent elevation discontinuity remains within the 3,000 m TEST bound;
9. invalid terrain classifications are rejected;
10. representative seeds contain at least one disconnected island landmass;
11. hydrology produces deterministic downstream links, drainage basins and rivers;
12. every river/downstream result is derived from canonical terrain rather than renderer state;
13. save v3 round-trips all authoritative terrain samples exactly;
14. save/load recomputation produces identical hydrology and landmass topology;
15. a valid terrain sample change changes canonical state bytes/checksum;
16. terrain and derived geography reproduce during command replay;
17. RenderSnapshot v3 carries terrain, hydrology and landmass arrays;
18. browser WASM initializes the authoritative engine with generated terrain;
19. Three.js builds the heightfield, sea plane and authoritative river network;
20. locked Rust CI, headless regression, WASM build, TypeScript and Vite production build all pass.
