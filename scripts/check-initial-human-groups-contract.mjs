import { readFileSync } from "node:fs";

const modelEntity = readFileSync("crates/simulation-model/src/entity.rs", "utf8");
const worldgenRoot = readFileSync("crates/simulation-worldgen/src/lib.rs", "utf8");
const generator = readFileSync("crates/simulation-worldgen/src/human_groups.rs", "utf8");
const save = readFileSync("crates/simulation-save/src/lib.rs", "utf8");
const binary = readFileSync("crates/simulation-save/src/binary.rs", "utf8");
const protocol = readFileSync("crates/simulation-protocol/src/lib.rs", "utf8");
const viewerTypes = readFileSync("viewer/src/types.ts", "utf8");
const generationTests = readFileSync(
  "crates/simulation-worldgen/tests/initial_human_groups.rs",
  "utf8",
);
const saveTests = readFileSync(
  "crates/simulation-save/tests/human_group_roundtrip.rs",
  "utf8",
);
const protocolTests = readFileSync(
  "crates/simulation-protocol/tests/human_group_snapshot.rs",
  "utf8",
);

for (const fragment of [
  "pub struct HumanGroupInitialState",
  "pub population: u64",
  "pub food_stock_person_days: u64",
  "pub basic_resource_stock_units: u64",
  "pub struct HumanGroupBehaviorProfile",
  "pub mobility_permille: u16",
  "pub exploration_permille: u16",
  "pub settlement_bias_permille: u16",
  "pub struct InitialKnowledgeProfile",
  "pub fn create_initialized",
]) {
  if (!modelEntity.includes(fragment)) {
    throw new Error(`Stage 2.6 model contract missing: ${fragment}`);
  }
}

for (const fragment of [
  "mod human_groups;",
  "generate_initial_human_groups",
  "InitialHumanGroupConfig",
  "PermilleRange",
]) {
  if (!worldgenRoot.includes(fragment) && !generator.includes(fragment)) {
    throw new Error(`Stage 2.6 worldgen contract missing: ${fragment}`);
  }
}

for (const fragment of [
  "pub group_count: usize",
  "pub total_population: u64",
  "pub contact_radius_m: u32",
  "RegionSurface::Land",
  "carrying_capacity_people_per_km2",
  "food_capacity_tonnes_per_year",
  "initial_group_contacts",
  "distribute_population",
]) {
  if (!generator.includes(fragment)) {
    throw new Error(`Stage 2.6 generator contract missing: ${fragment}`);
  }
}

for (const forbidden of [
  "const HUMAN_GROUP_COUNT",
  "const INITIAL_GROUP_COUNT",
  "CreateCountryEntity",
  "PromotePoliticalEntityToCountry",
]) {
  if (generator.includes(forbidden)) {
    throw new Error(`Stage 2.6 generator must not pre-create state entities: ${forbidden}`);
  }
}

const saveVersion = save.match(/pub const SAVE_FORMAT_VERSION: u32 = (\d+)/);
if (saveVersion === null || Number(saveVersion[1]) < 6) {
  throw new Error("Stage 2.6 requires save format v6 or newer");
}
for (const fragment of [
  "write_human_group",
  "read_human_group",
  "HumanGroup initial state references invalid geography",
  "knowledge_seed_values",
]) {
  if (!binary.includes(fragment) && !save.includes(fragment)) {
    throw new Error(`Stage 2.6 persistence contract missing: ${fragment}`);
  }
}

const renderVersion = protocol.match(/pub const RENDER_SNAPSHOT_VERSION: u32 = (\d+)/);
if (renderVersion === null || Number(renderVersion[1]) < 5) {
  throw new Error("Stage 2.6 requires RenderSnapshot v5 or newer");
}
for (const fragment of [
  "pub struct RenderHumanGroupSnapshot",
  "pub human_groups: Vec<RenderHumanGroupSnapshot>",
  "id: group.id.0.to_string()",
  "region_id: initial.region_id.0.to_string()",
]) {
  if (!protocol.includes(fragment)) {
    throw new Error(`Stage 2.6 render contract missing: ${fragment}`);
  }
}
for (const fragment of [
  "export interface RenderHumanGroupSnapshot",
  "humanGroups: RenderHumanGroupSnapshot[]",
]) {
  if (!viewerTypes.includes(fragment)) {
    throw new Error(`Stage 2.6 TypeScript contract missing: ${fragment}`);
  }
}

for (const fragment of [
  "group_count_is_parameterized_and_total_population_is_exact",
  "groups_use_distinct_habitable_regions_and_valid_initial_stocks",
  "same_seed_reproduces_groups_while_other_seed_changes_initialization",
  "contact_matrix_covers_every_pair_and_uses_distance_threshold",
]) {
  if (!generationTests.includes(fragment)) {
    throw new Error(`Stage 2.6 generation acceptance missing: ${fragment}`);
  }
}
for (const fragment of [
  "initialized_human_groups_round_trip_exactly_in_save_v6",
  "initial_human_group_state_changes_canonical_binary",
]) {
  if (!saveTests.includes(fragment)) {
    throw new Error(`Stage 2.6 save acceptance missing: ${fragment}`);
  }
}
if (!protocolTests.includes("zero_country_world_exposes_initialized_human_group_snapshot")) {
  throw new Error("Stage 2.6 RenderSnapshot acceptance missing");
}

console.log(
  `Stage 2.6 initial HumanGroup contract verified on save v${saveVersion[1]} / render v${renderVersion[1]}`,
);
