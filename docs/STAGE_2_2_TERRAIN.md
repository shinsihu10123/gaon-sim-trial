# Stage 2.2 — Deterministic Terrain and Elevation

## Purpose

Stage 2.2 gives the TEST world an authoritative continuous terrain surface generated from the world seed and rendered as actual 3D geometry.

The terrain is simulation state, not decorative renderer data. Elevation and terrain classification must therefore survive save/restore, alter the authoritative digest and regenerate deterministically during replay.

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
TerrainState
   ↓
WorldState
   ↓
canonical save / digest
   ↓
RenderSnapshot
   ↓
Three.js terrain mesh
```

This separation allows later region generation, resources and settlement placement to reuse a deterministic world-generation layer without coupling generation to tick processing.

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

## Moisture

Moisture uses a separate deterministic noise field with broad and regional scales. Values are normalized to integer permille.

This is a Stage 2.2 environmental substrate, not a completed climate simulation. Latitude, prevailing winds, rain shadows and seasonal climate remain future extensions if required.

## Relief classification

Relief is derived from elevation and immediate coastal adjacency:

- `DeepOcean`: below -1,200 m
- `ShallowOcean`: -1,200 m to below sea level
- `Coast`: land sample adjacent to an ocean sample
- `Plains`: inland land below 450 m
- `Hills`: 450 m to below 1,500 m
- `Mountains`: 1,500 m and above

Relief and biome remain separate concepts so later economic/military effects can use terrain geometry independently from ecological cover.

## Biome classification

Stage 2.2 uses a compact deterministic classification:

- `Ocean`
- `Grassland`
- `Forest`
- `Desert`
- `Wetland`
- `Alpine`

Classification depends on elevation and moisture. It is intentionally not a full Köppen climate model.

## Persistence and authoritative digest

Terrain expands authoritative state, so save format advances to **v3**.

The canonical binary stores:

- terrain bounds
- width and height
- spacing
- sea level
- every sample elevation
- every sample moisture value
- relief code
- biome code

`authoritative_state_digest()` hashes the same canonical binary substrate. Changing even one valid terrain sample therefore changes the canonical state bytes and digest substrate.

## Replay semantics

Stage 1 command-journal replay reconstructs dynamic command/event history from a seed and accepted command journal.

From Stage 2 onward, deterministic static world-generation state must also be reconstructed from the same seed. The Stage 2.2 headless path therefore:

1. replays the command journal;
2. regenerates terrain from the original seed;
3. inserts it into the replay snapshot;
4. validates the snapshot through save v3;
5. compares the complete authoritative state and digest.

This prevents generated geography from becoming an untracked external dependency.

## RenderSnapshot v2

Renderer protocol version advances to v2 and includes:

- terrain dimensions
- spacing and sea level
- elevation array
- moisture array
- relief-code array
- biome-code array

No independently generated terrain exists in TypeScript.

## Three.js representation

The Viewer builds an indexed `BufferGeometry` directly from authoritative elevation samples.

- 16,641 terrain vertices
- two triangles per heightfield cell
- computed vertex normals
- biome/relief vertex colors
- separate sea-level plane
- presentation-only vertical exaggeration

The terrain mesh and region-topology layer remain separate scene objects. Later borders, cities, roads, resources and armies can therefore be overlaid without replacing terrain state.

## Deferred scope

Stage 2.2 does not yet implement:

- river network or hydrology
- erosion simulation
- detailed climate circulation
- terrain economic modifiers
- resource deposits
- final 60-region generation on the terrain
- country borders or cities
- selectable region interaction

These belong to later Stage 2 subsections.

## Acceptance criteria

Stage 2.2 is PASS only when all of the following hold:

1. 129×129 canonical terrain validates;
2. same seed generates identical terrain;
3. different seeds produce different terrain;
4. canonical seed has a balanced land/ocean distribution;
5. all outer-edge samples are ocean;
6. required relief classes are generated;
7. required biome classes are generated;
8. adjacent elevation discontinuity remains within the TEST validation bound;
9. invalid terrain classifications are rejected;
10. save v3 round-trips all terrain samples exactly;
11. a valid terrain sample change changes canonical state bytes/checksum;
12. terrain is regenerated during deterministic command replay;
13. RenderSnapshot v2 carries the authoritative terrain arrays;
14. browser WASM initializes the authoritative engine with generated terrain;
15. Three.js builds a heightfield mesh and separate sea level from the snapshot;
16. locked Rust CI, headless regression, WASM build, TypeScript and Vite production build all pass.
