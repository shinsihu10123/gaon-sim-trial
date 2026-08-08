import { readFileSync } from "node:fs";

const bootstrap = readFileSync("crates/simulation-worldgen/src/bootstrap.rs", "utf8");
const worldgen = readFileSync("crates/simulation-worldgen/src/lib.rs", "utf8");
const lifecycle = readFileSync("crates/simulation-core/src/entity_lifecycle.rs", "utf8");
const world = readFileSync("crates/simulation-model/src/world.rs", "utf8");
const protocol = readFileSync("crates/simulation-protocol/src/lib.rs", "utf8");
const wasm = readFileSync("crates/simulation-wasm/src/lib.rs", "utf8");
const runner = readFileSync("crates/simulation-runner/src/main.rs", "utf8");
const bridge = readFileSync("viewer/src/simulation-bridge.ts", "utf8");
const terrain = readFileSync("viewer/src/terrain-layer.ts", "utf8");
const topology = readFileSync("viewer/src/world-layer.ts", "utf8");
const selection = readFileSync("viewer/src/entity-selection.ts", "utf8");
const types = readFileSync("viewer/src/types.ts", "utf8");
const main = readFileSync("viewer/src/main.ts", "utf8");

for (const fragment of [
  "generate_trial_civilization_origin_world",
  "trial_preview_human_group_config",
  "BENCHMARK_REGION_COLUMNS",
  "BENCHMARK_REGION_ROWS",
]) {
  if (!bootstrap.includes(fragment) || !worldgen.includes("mod bootstrap;")) {
    throw new Error(`shared bootstrap contract missing: ${fragment}`);
  }
}
if (!wasm.includes("generate_trial_civilization_origin_world") || !runner.includes("generate_trial_civilization_origin_world")) {
  throw new Error("WASM/headless do not share civilization-origin bootstrap");
}
if (!lifecycle.includes("initial.region_id == region_id")) {
  throw new Error("HumanGroup Region reference is not protected during removal");
}
for (const fragment of ["InvalidBoundaryGeometry", "CenterOutsideBoundary", "boundary_self_intersects", "point_in_polygon_or_boundary"]) {
  if (!world.includes(fragment)) throw new Error(`Region geometry invariant missing: ${fragment}`);
}
for (const fragment of [
  "pub fn from_world_dynamic",
  "pub settlements: Vec<RenderIdentitySnapshot>",
  "pub countries: Vec<RenderIdentitySnapshot>",
  "pub cities: Vec<RenderCitySnapshot>",
  "population: state.population.to_string()",
  "region_id: state.region_id.0.to_string()",
  "nutrition_permille: state.nutrition_permille",
  "cohesion_permille: state.cohesion_permille",
  "risk_permille: state.risk_permille",
]) {
  if (!protocol.includes(fragment)) throw new Error(`render protocol hardening missing: ${fragment}`);
}
for (const fragment of ["staticTerrain", "dynamic snapshot arrived before static terrain initialization"]) {
  if (!bridge.includes(fragment)) throw new Error(`static snapshot cache missing: ${fragment}`);
}
if (!terrain.includes("this.renderedTerrain === terrain")) {
  throw new Error("TerrainLayer does not retain immutable geometry");
}
if (!topology.includes("private readonly visuals = new Map")) {
  throw new Error("Region topology is not reconciled by stable ID");
}
for (const fragment of ["previousSelectedKey", "private readonly proxies = new Map", "this.renderHighlight(target)"]) {
  if (!selection.includes(fragment)) throw new Error(`selection retention missing: ${fragment}`);
}
for (const fragment of [
  "population: string",
  "foodStockPersonDays: string",
  "nutritionPermille: number",
  "cohesionPermille: number",
  "riskPermille: number",
  "countries: RenderIdentitySnapshot[]",
  "cities: RenderCitySnapshot[]",
]) {
  if (!types.includes(fragment)) throw new Error(`Viewer exact/dynamic type missing: ${fragment}`);
}
if (!main.includes("export async function advanceViewerDays") || !main.includes("applySnapshot(snapshot)")) {
  throw new Error("Viewer has no reusable dynamic snapshot application path");
}

console.log("Stage 2.8.0 integration/render hardening contract verified with Stage 3.1 runtime state");
