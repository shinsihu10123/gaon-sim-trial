# Stage 2.7 Camera and Selection

Status: **IN PROGRESS — PERMANENT CI VALIDATION**

Authoritative scope: WBS v3.1 civilization-origin revision.

## WBS v3.1 scope

- ◐ 2.7.1 world-scale camera
- ◐ 2.7.2 zoom / rotate / move
- ◐ 2.7.3 camera range limits
- ◐ 2.7.4 HumanGroup / Settlement / Community / PoliticalEntity / Country / Region selection contract
- ◐ 2.7.5 selected-target highlight
- ◐ 2.7.6 selected-entity focus and world framing
- ◐ 2.7.7 spatial-event automatic focus hook for war / mass migration

No item is complete until implementation, permanent Viewer validation and evidence all exist.

## Current implementation

- shared `WorldSpaceTransform` used by terrain, Region topology, selection and camera focus
- `WorldCameraController` over Three.js `OrbitControls`
- zoom, rotate and pan enabled
- minimum/maximum distance and polar-angle limits
- world-bound target clamping
- world framing and point focus
- transparent Region and HumanGroup selection hit proxies
- ray-cast click selection
- double-click selection + focus
- entity dropdown / focus / world controls
- Region outline highlight and point reticle highlight
- selection type contract covering HumanGroup, Settlement, Community, PoliticalEntity, Country and Region
- Viewer-side war / mass-migration spatial auto-focus interface
- permanent Stage 2.7 source contract chained into Viewer CI
- built Viewer artifact retention in PR CI
- `develop` GitHub Pages preview deployment workflow with artifact fallback

## Explicit non-goals

- no final HumanGroup 3D model
- no Settlement/City/Country visual model
- no political territory shading or dynamic borders
- no roads, trade, military or battle render
- no domain war/migration generation

## Current integration gate

A Draft PR will validate the branch through the permanent Rust/WASM/TypeScript/Vite/Viewer contract path. Stage 2.7 remains in progress until the cleaned final head is green and evidence is recorded.
