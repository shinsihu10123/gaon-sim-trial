import { readFileSync } from "node:fs";

const model = readFileSync("crates/simulation-model/src/world.rs", "utf8");
const modelRoot = readFileSync("crates/simulation-model/src/lib.rs", "utf8");
const save = readFileSync("crates/simulation-save/src/lib.rs", "utf8");
const binary = readFileSync("crates/simulation-save/src/binary.rs", "utf8");
const protocol = readFileSync("crates/simulation-protocol/src/lib.rs", "utf8");

for (const fragment of [
  "pub const TRIAL_REGION_COUNT: usize = 60",
  "pub struct MapPoint",
  "pub struct WorldBounds",
  "pub enum RegionSurface",
  "pub struct RegionState",
  "pub legal_owner: Option<CountryId>",
  "pub controller: Option<CountryId>",
  "pub struct WorldSpatialState",
  "regions.len() != TRIAL_REGION_COUNT",
  "AsymmetricAdjacency",
]) {
  if (!model.includes(fragment)) {
    throw new Error(`Stage 2.1 model contract missing: ${fragment}`);
  }
}

if (!modelRoot.includes("pub spatial: WorldSpatialState")) {
  throw new Error("WorldState does not own WorldSpatialState");
}
if (!save.includes("pub const SAVE_FORMAT_VERSION: u32 = 2")) {
  throw new Error("save format was not advanced for Stage 2.1 topology");
}
for (const fragment of ["write_region", "read_region", "write_optional_country", "read_optional_country"]) {
  if (!binary.includes(fragment)) {
    throw new Error(`world persistence contract missing: ${fragment}`);
  }
}
for (const fragment of [
  "pub struct RenderWorldSnapshot",
  "pub struct RenderRegionSnapshot",
  "legal_owner: Option<u16>",
  "controller: Option<u16>",
]) {
  if (!protocol.includes(fragment)) {
    throw new Error(`render protocol contract missing: ${fragment}`);
  }
}

console.log("Stage 2.1 world data contract verified");
