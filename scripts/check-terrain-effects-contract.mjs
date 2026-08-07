import { readFileSync } from "node:fs";

const effects = readFileSync("crates/simulation-model/src/terrain_effects.rs", "utf8");
const modelRoot = readFileSync("crates/simulation-model/src/lib.rs", "utf8");
const generatedTests = readFileSync(
  "crates/simulation-worldgen/tests/terrain_effects.rs",
  "utf8",
);

for (const fragment of [
  "pub struct TerrainEffects",
  "agriculture_yield_permille",
  "construction_cost_permille",
  "movement_cost_permille",
  "defense_multiplier_permille",
  "port_feasibility_permille",
  "carrying_capacity_people_per_km2",
  "productivity_multiplier_permille",
  "pub struct TerrainEffectField",
  "aggregate_indices",
  "derive_terrain_effects",
  "terrain.derive_hydrology()",
  "is_river_mouth",
]) {
  if (!effects.includes(fragment)) {
    throw new Error(`Stage 2.3 terrain effect contract missing: ${fragment}`);
  }
}

for (const fragment of [
  "derive_terrain_effects",
  "TerrainEffectField",
  "TerrainEffects",
]) {
  if (!modelRoot.includes(fragment)) {
    throw new Error(`Stage 2.3 public API missing: ${fragment}`);
  }
}

for (const fragment of [
  "effect_field_is_deterministic_and_aligned_to_terrain",
  "ocean_has_no_land_economic_or_military_effects",
  "generated_world_exposes_nontrivial_effect_ranges",
  "mountains_trade_accessibility_for_defense",
  "aggregation_produces_region_ready_average",
]) {
  if (!generatedTests.includes(fragment)) {
    throw new Error(`Stage 2.3 generated-world validation missing: ${fragment}`);
  }
}

console.log("Stage 2.3 terrain simulation-effect contract verified");
