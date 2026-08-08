import { readFileSync } from "node:fs";

const model = readFileSync("crates/simulation-model/src/world.rs", "utf8");
const modelRoot = readFileSync("crates/simulation-model/src/lib.rs", "utf8");
const save = readFileSync("crates/simulation-save/src/lib.rs", "utf8");
const binary = readFileSync("crates/simulation-save/src/binary.rs", "utf8");
const protocol = readFileSync("crates/simulation-protocol/src/lib.rs", "utf8");

for (const fragment of [
  "pub const TRIAL_REGION_COUNT: usize = 60",
  "Standard Benchmark S fixture only",
  "pub struct MapPoint",
  "pub struct WorldBounds",
  "pub enum RegionSurface",
  "pub struct RegionState",
  "pub legal_owner: Option<CountryId>",
  "pub controller: Option<CountryId>",
  "pub struct WorldSpatialState",
  "pub fn new(bounds: WorldBounds, regions: Vec<RegionState>)",
  "binary_search_by_key",
  "AsymmetricAdjacency",
]) {
  if (!model.includes(fragment)) {
    throw new Error(`Stage 2.1 model contract missing: ${fragment}`);
  }
}

for (const forbidden of [
  "regions.len() != TRIAL_REGION_COUNT",
  "usize::from(neighbor.0) > TRIAL_REGION_COUNT",
  "self.regions.get(usize::from(id.0 - 1))",
  "must contain exactly 60",
  "IDs 1..=60",
]) {
  if (model.includes(forbidden)) {
    throw new Error(`fixed-region core invariant still present: ${forbidden}`);
  }
}

if (!modelRoot.includes("pub spatial: WorldSpatialState")) {
  throw new Error("WorldState does not own WorldSpatialState");
}
const versionMatch = save.match(/pub const SAVE_FORMAT_VERSION: u32 = (\d+)/);
if (versionMatch === null || Number(versionMatch[1]) < 2) {
  throw new Error("Stage 2.1 requires a versioned save format v2 or newer");
}
for (const fragment of [
  "write_count(bytes, \"regions\", world.spatial.regions.len())",
  "let region_count = cursor.read_count(\"regions\")",
  "write_region",
  "read_region",
  "write_optional_country",
  "read_optional_country",
]) {
  if (!binary.includes(fragment)) {
    throw new Error(`world persistence contract missing: ${fragment}`);
  }
}
for (const fragment of [
  "pub struct RenderWorldSnapshot",
  "pub struct RenderRegionSnapshot",
  "regions: world",
  ".regions",
  ".iter()",
  "legal_owner: Option<u16>",
  "controller: Option<u16>",
]) {
  if (!protocol.includes(fragment)) {
    throw new Error(`render protocol contract missing: ${fragment}`);
  }
}

console.log(
  `Stage 2.1 dynamic-region contract verified on save format v${versionMatch[1]}`,
);
