# Stage 2.8 Dynamic 3D Visualization

## Scope

This stage turns the Stage 2.7 inspection Viewer into a dynamic world-state visualization layer without moving authority out of the Rust Simulation Core.

The v3.1 WBS defines 14 items:

1. terrain materials and elevation
2. basic HumanGroup markers
3. basic movement-route visualization
4. temporary camp / Settlement model foundation
5. basic resource-use-area visualization
6. political influence / Country territory rendering foundation
7. Region boundaries
8. capital / City model extension foundation
9. agricultural / production-intensification rendering foundation
10. mine / resource-facility extension foundation
11. port / road / railway extension foundation
12. Simulation State to 3D-object binding
13. runtime Entity create/remove reflection
14. variable Entity collections in RenderSnapshot

## First implementation slice

This branch implements the first visible Stage 2.8 slice:

- 2.8.1: authoritative terrain remains a real heightfield, while material appearance now combines biome, moisture, relief and altitude; water uses a dedicated physical surface and rivers remain state-derived overlays.
- 2.8.2: every RenderSnapshot HumanGroup receives a persistent, visible 3D marker keyed by Stable Entity ID. Marker position follows authoritative x/z coordinates and terrain elevation; size is derived from population and appearance only uses already-authoritative behavior fields.
- 2.8.7: existing Region boundaries are retained but visually reduced so topology remains inspectable without dominating the terrain.
- 2.8.12-2.8.14: the Stage 2.8.0 hardening remains the binding foundation: Viewer objects are derived from Simulation RenderSnapshot state, Entity objects are retained by Stable ID and removed when the authoritative collection removes them, and variable Entity collections are present in RenderSnapshot v6.

## Authority rules

- Viewer objects never mutate Simulation State.
- Visual scale, color and opacity are presentation-only transforms.
- Stable Entity ID is the retained-render identity key.
- HumanGroup position, population and behavior values come only from RenderSnapshot.
- No Settlement, Country, City, road, industry or movement outcome is invented by the Viewer.

## Explicit non-goals of this slice

Movement routes, Settlement visuals, resource-use footprints, political influence, Country territory fills, City models, agriculture, mines, ports, roads and railways remain future Stage 2.8 work and must wait for authoritative state fields or lifecycle events that can support them.
