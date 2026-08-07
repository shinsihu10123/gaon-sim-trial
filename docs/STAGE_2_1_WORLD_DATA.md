# Stage 2.1 — World Data Structure

## Purpose

Stage 2.1 establishes the authoritative spatial identity of the TEST v0.1 world before terrain generation, resources, cities or countries are populated.

This stage does **not** generate a finished map. It defines the data contract that later Stage 2 systems must populate without changing region identity or state ownership rules.

## Baseline correction discovered at Stage 2 entry

The Stage 1 acceptance document stated that the TypeScript/Vite/Three.js Viewer and Rust WASM bridge existed, but a direct repository inspection at Stage 2 entry showed that PR #6 had merged only workflow files. The actual `viewer`, `simulation-protocol` and `simulation-wasm` source trees were absent from `main`.

Stage 2.1 therefore restores those missing Stage 1.1 source files before accepting new world data work. The corrected browser architecture is:

`SimulationEngine -> RenderSnapshot -> simulation-wasm -> Web Worker -> SimulationBridge -> Three.js`

The Web Worker owns the Rust engine. The main thread and renderer only receive snapshots.

## Authoritative spatial model

The TEST world uses exactly **60 regions** after initialization.

### Stable identity

- `RegionId` values are canonical `1..=60`.
- Region vector order must match RegionId order.
- Region identity never depends on renderer object order.

### Coordinates

Authoritative horizontal coordinates use signed 32-bit integer metres:

- `MapPoint.x_m`
- `MapPoint.z_m`

Three.js may convert these values to floating point for display, but floating renderer coordinates are not authoritative simulation state.

### World bounds

`WorldBounds` defines one continuous rectangular coordinate domain. Region centers and boundary points must lie within those bounds.

### Region topology

Each `RegionState` contains:

- stable RegionId
- `Land` or `Ocean` surface class
- center point
- ordered polygon boundary
- strictly ascending unique neighbor RegionIds
- political ownership/control state

Adjacency is required to be reciprocal. A region cannot reference itself or an unknown region.

Polygon overlap, terrain elevation and hydrology are intentionally outside Stage 2.1 and are handled by later Stage 2 work.

## Ownership and control

`legal_owner` and `controller` are separate fields.

- `legal_owner`: treaty-recognized sovereignty
- `controller`: current effective administrative/military controller

They may differ during future occupation. This prevents a temporary military occupation from immediately becoming a legal border change.

At Stage 2.1, ocean regions have neither field set.

## Initialization state

Stage 1 worlds remain valid as one canonical uninitialized representation:

- `bounds = None`
- `regions = []`

Once world generation is introduced, an initialized world must contain exactly 60 validated regions.

No placeholder or fake region geometry is created merely to make the Viewer look populated.

## Persistence and determinism

Adding spatial topology changes authoritative state, so the canonical save format advances from v1 to **v2**.

The save binary includes:

- world bounds
- every RegionId
- land/ocean classification
- center coordinates
- complete polygon boundaries
- adjacency lists
- legal owner
- controller

The authoritative state digest continues to hash the canonical save binary, therefore any spatial change also changes the digest.

## RenderSnapshot contract

The renderer receives spatial state through `simulation-protocol` only.

A region snapshot exposes:

- RegionId
- surface class
- center
- boundary
- neighbors
- legal owner
- controller

The Viewer may render these as topology outlines. It must not mutate them or own an alternative simulation copy.

## Acceptance criteria

Stage 2.1 is PASS only when all of the following are verified:

1. canonical uninitialized Stage 1 world remains valid;
2. exactly 60 canonical region records validate;
3. invalid region counts fail;
4. non-reciprocal adjacency fails;
5. self/unknown/unsorted neighbors fail;
6. region points outside world bounds fail;
7. owner and controller remain separate fields;
8. save v2 round-trips an initialized 60-region world exactly;
9. authoritative digest includes spatial state through the save binary;
10. RenderSnapshot exposes world bounds and all region topology fields;
11. the browser engine remains owned inside a Web Worker;
12. Rust workspace, WASM build, Viewer typecheck and Viewer production build pass.
