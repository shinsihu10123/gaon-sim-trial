import { readFileSync } from "node:fs";

const modelRoot = readFileSync("crates/simulation-model/src/lib.rs", "utf8");
const resources = readFileSync("crates/simulation-model/src/resources.rs", "utf8");
const worldgenRoot = readFileSync("crates/simulation-worldgen/src/lib.rs", "utf8");
const bootstrap = readFileSync("crates/simulation-worldgen/src/bootstrap.rs", "utf8");
const generator = readFileSync("crates/simulation-worldgen/src/resources.rs", "utf8");
const save = readFileSync("crates/simulation-save/src/lib.rs", "utf8");
const binary = readFileSync("crates/simulation-save/src/binary.rs", "utf8");
const wasm = readFileSync("crates/simulation-wasm/src/lib.rs", "utf8");
const runner = readFileSync("crates/simulation-runner/src/main.rs", "utf8");
const roundTrip = readFileSync("crates/simulation-save/tests/resource_roundtrip.rs", "utf8");

for (const fragment of ["mod resources;", "pub resources: Option<ResourceFieldState>"]) {
  if (!modelRoot.includes(fragment)) {
    throw new Error(`Stage 2.4 model contract missing: ${fragment}`);
  }
}

for (const fragment of [
  "pub struct ResourceDeposit",
  "pub fn extract",
  "pub struct ResourceFieldState",
  "validate_against_terrain",
  "food_capacity_tonnes_per_year",
  "remaining_quantity",
  "quality_permille",
  "accessibility_permille",
]) {
  if (!resources.includes(fragment)) {
    throw new Error(`Stage 2.4 resource-state contract missing: ${fragment}`);
  }
}

for (const fragment of [
  "generate_trial_resources",
  "validate_resource_balance",
  "ResourceBalanceReport",
  "energy_deposit",
  "metal_deposit",
  "construction_deposit",
]) {
  if (!generator.includes(fragment) && !worldgenRoot.includes(fragment)) {
    throw new Error(`Stage 2.4 generation contract missing: ${fragment}`);
  }
}

const versionMatch = save.match(/pub const SAVE_FORMAT_VERSION: u32 = (\d+)/);
if (versionMatch === null || Number(versionMatch[1]) < 4) {
  throw new Error("Stage 2.4 requires save format v4 or newer");
}
for (const fragment of ["write_resource_field", "read_resource_field", "remaining_quantity"]) {
  if (!binary.includes(fragment)) {
    throw new Error(`Stage 2.4 binary contract missing: ${fragment}`);
  }
}

if (
  !bootstrap.includes("generate_trial_resources(seed, &terrain)") ||
  !bootstrap.includes("world.resources = Some(resources)")
) {
  throw new Error("shared civilization bootstrap does not create authoritative resources");
}
for (const [name, source] of [["WASM", wasm], ["headless", runner]]) {
  if (!source.includes("generate_trial_civilization_origin_world")) {
    throw new Error(`${name} initialization does not use shared authoritative world bootstrap`);
  }
}

for (const fragment of ["extract", "state_binary", "decode_bundle", "assert_eq!(restored.world, snapshot.world)"]) {
  if (!roundTrip.includes(fragment)) {
    throw new Error(`Stage 2.4 round-trip contract missing: ${fragment}`);
  }
}

console.log(`Stage 2.4 resource contract verified on save format v${versionMatch[1]}`);
