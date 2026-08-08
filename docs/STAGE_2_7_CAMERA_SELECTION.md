# Stage 2.7 Camera and Selection

## Purpose

Stage 2.7 turns the existing Three.js world into an inspectable simulation workspace without pre-implementing Stage 2.8 visual models.

The stage provides camera navigation, selection hit proxies, highlighting and spatial focus against authoritative RenderSnapshot coordinates.

## Camera

`WorldCameraController` owns Viewer camera navigation rules on top of Three.js `OrbitControls`.

Supported interaction:

- world framing
- zoom
- rotate
- pan/move
- target clamping to world bounds
- minimum and maximum camera distance
- polar-angle limits
- focus on a world coordinate or scene coordinate

The camera never writes Simulation Core state.

## Shared world-space transform

Terrain, Region boundaries, selection and camera focus use the same `WorldSpaceTransform`.

This prevents different Viewer layers from inventing separate world-to-scene scaling rules.

## Entity selection

The selection contract accepts these civilization-origin entity kinds:

- HumanGroup
- Settlement
- Community
- PoliticalEntity
- Country
- Region

The current RenderSnapshot contains selectable Region and HumanGroup records. Later entity kinds can enter the same selection contract when their render records exist.

Selection hit testing uses transparent Viewer-only proxy geometry. These proxies are not simulation entities and are not authoritative state.

## Highlighting

Selected Regions receive a separate outline highlight.

Selected point-like entities receive a Viewer-only reticle.

Stage 2.7 deliberately does not introduce final HumanGroup, Settlement, City or Country visual models; those belong to Stage 2.8 and later visualization stages.

## Focus

Users can:

- select through the entity panel
- click a selectable object
- double-click to select and focus
- request focus from the panel
- return to the world framing view

## Spatial event focus

`SpatialEventFocusController` provides the Viewer-side automatic focus contract for spatially important events.

The initial supported categories are:

- war
- mass migration

The controller accepts spatial coordinates and moves the camera to the event location. Stage 2.7 provides the focus mechanism; later war/migration stages provide real event sources.

## Preview delivery

The Viewer CI retains the built `viewer/dist` directory as a downloadable workflow artifact.

After Stage 2.7 is merged into `develop`, the `Viewer Preview` workflow rebuilds the actual Rust WASM + TypeScript/Three.js Viewer and deploys `viewer/dist` to GitHub Pages. The deployment workflow also retains the same `dist` output as a fallback artifact.

## Non-goals

Stage 2.7 does not implement:

- final HumanGroup 3D models
- Settlement/City visual models
- Country territory shading
- dynamic borders
- roads, railways or ports
- trade animation
- armies or battle visualization
- simulation-domain war or migration generation
