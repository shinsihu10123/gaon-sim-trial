import { readFileSync } from "node:fs";

const terrain = readFileSync("crates/simulation-model/src/terrain.rs", "utf8");
const modelRoot = readFileSync("crates/simulation-model/src/lib.rs", "utf8");
const worldgen = readFileSync("crates/simulation-worldgen/src/lib.rs", "utf8");
const save = readFileSync("crates/simulation-save/src/lib.rs", "utf8");
const binary = readFileSync("crates/simulation-save/src/binary.rs", "utf8");
const protocol = readFileSync("crates/simulation-protocol/src/lib.rs", "utf8");
const wasm = readFileSync("crates/simulation-wasm/src/lib.rs", "utf8");
const viewer = readFileSync("viewer/src/terrain-layer.ts", "utf8");

for (const fragment of [
  "pub const TERRAIN_GRID_SIDE: u16 = 129",
  "pub const TERRAIN_GRID_SPACING_M: i32 = 10_000",
  "pub enum ReliefClass",
  "DeepOcean",
  "ShallowOcean",
  "Coast",
  "Plains",
  "Hills",
  "Mountains",
  "pub enum BiomeClass",
  "Forest",
  "Desert",
  "Wetland",
  "pub struct TerrainState",
]) {
  if (!terrain.includes(fragment)) {
    throw new Error(`Stage 2.2 terrain model contract missing: ${fragment}`);
  }
}

if (!modelRoot.includes("pub terrain: Option<TerrainState>")) {
  throw new Error("WorldState does not own authoritative TerrainState");
}
for (const fragment of [
  "pub fn generate_trial_terrain",
  "value_noise",
  "raw_continental_signal",
  "edge_distance",
  "classify_relief",
  "classify_biome",
]) {
  if (!worldgen.includes(fragment)) {
    throw new Error(`Stage 2.2 worldgen contract missing: ${fragment}`);
  }
}
const versionMatch = save.match(/pub const SAVE_FORMAT_VERSION: u32 = (\d+)/);
if (versionMatch === null || Number(versionMatch[1]) < 3) {
  throw new Error("Stage 2.2 requires save format version 3 or newer");
}
for (const fragment of [
  "write_terrain",
  "read_terrain",
  "write_relief",
  "read_relief",
  "write_biome",
  "read_biome",
]) {
  if (!binary.includes(fragment)) {
    throw new Error(`Stage 2.2 terrain persistence contract missing: ${fragment}`);
  }
}
for (const fragment of [
  "pub struct RenderTerrainSnapshot",
  "elevation_m: Vec<i16>",
  "moisture_permille: Vec<u16>",
  "relief_codes: Vec<u8>",
  "biome_codes: Vec<u8>",
]) {
  if (!protocol.includes(fragment)) {
    throw new Error(`Stage 2.2 render terrain contract missing: ${fragment}`);
  }
}
if (!wasm.includes("generate_trial_terrain(seed)")) {
  throw new Error("browser authoritative engine is not bootstrapped with generated terrain");
}
for (const fragment of [
  "authoritative-terrain-heightfield",
  "authoritative-sea-level",
  "computeVertexNormals",
]) {
  if (!viewer.includes(fragment)) {
    throw new Error(`Three.js terrain layer contract missing: ${fragment}`);
  }
}

console.log(`Stage 2.2 terrain contract verified on save format v${versionMatch[1]}`);
