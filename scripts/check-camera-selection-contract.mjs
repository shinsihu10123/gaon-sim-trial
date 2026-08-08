import { readFileSync } from "node:fs";

const camera = readFileSync("viewer/src/world-camera.ts", "utf8");
const selection = readFileSync("viewer/src/entity-selection.ts", "utf8");
const panel = readFileSync("viewer/src/selection-panel.ts", "utf8");
const focus = readFileSync("viewer/src/event-focus.ts", "utf8");
const main = readFileSync("viewer/src/main.ts", "utf8");
const space = readFileSync("viewer/src/world-space.ts", "utf8");
const terrain = readFileSync("viewer/src/terrain-layer.ts", "utf8");
const topology = readFileSync("viewer/src/world-layer.ts", "utf8");

for (const fragment of [
  "controls.enableZoom = true",
  "controls.enableRotate = true",
  "controls.enablePan = true",
  "controls.minDistance",
  "controls.maxDistance",
  "controls.minPolarAngle",
  "controls.maxPolarAngle",
  "clampSceneTarget",
  "frameWorld()",
  "focusWorldPoint",
  "focusScenePoint",
]) {
  if (!camera.includes(fragment)) {
    throw new Error(`Stage 2.7 camera contract missing: ${fragment}`);
  }
}

for (const kind of [
  '"human_group"',
  '"settlement"',
  '"community"',
  '"political_entity"',
  '"country"',
  '"region"',
]) {
  if (!selection.includes(kind)) {
    throw new Error(`Stage 2.7 selectable kind missing: ${kind}`);
  }
}

for (const fragment of [
  "Raycaster",
  "pickFromClientPoint",
  "intersectObjects",
  "region-hit-",
  "human-group-hit-",
  "selected-region-",
  "selected-entity-reticle",
]) {
  if (!selection.includes(fragment)) {
    throw new Error(`Stage 2.7 selection/highlight contract missing: ${fragment}`);
  }
}

for (const fragment of ["Entity Selection", "Focus", "World", "setTargets", "setSelected"]) {
  if (!panel.includes(fragment)) {
    throw new Error(`Stage 2.7 selection panel contract missing: ${fragment}`);
  }
}

for (const fragment of ['"war"', '"mass_migration"', "focusWorldPoint", "lastFocusedEvent"]) {
  if (!focus.includes(fragment)) {
    throw new Error(`Stage 2.7 spatial event focus contract missing: ${fragment}`);
  }
}

for (const fragment of [
  'addEventListener("pointerdown"',
  'addEventListener("pointerup"',
  'addEventListener("dblclick"',
  "cameraController.frameWorld",
  "autoFocusSpatialEvent",
]) {
  if (!main.includes(fragment)) {
    throw new Error(`Stage 2.7 viewer integration contract missing: ${fragment}`);
  }
}

for (const fragment of ["WorldSpaceTransform", "worldPointToScene", "elevationToSceneY"]) {
  if (!space.includes(fragment)) {
    throw new Error(`Stage 2.7 world-space contract missing: ${fragment}`);
  }
}
if (!terrain.includes('from "./world-space"') || !topology.includes('from "./world-space"')) {
  throw new Error("terrain/topology do not share the Stage 2.7 world-space transform");
}

console.log("Stage 2.7 camera, selection and spatial-focus contract verified");
