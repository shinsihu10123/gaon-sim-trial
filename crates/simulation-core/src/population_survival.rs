use std::collections::BTreeMap;

use simulation_model::{
    derive_terrain_effects, EntityRegistry, HumanGroupId, HumanGroupRegistry, RegionId,
    TerrainState, WorldState,
};

const RATE_DENOMINATOR_PER_DAY: u128 = 365_000;
const BASE_BIRTH_RATE_PER_THOUSAND_YEAR: u32 = 38;
const BASE_DEATH_RATE_PER_THOUSAND_YEAR: u32 = 35;
const MAX_RISK_DEATH_RATE_PER_THOUSAND_YEAR: u32 = 6;
const MALNUTRITION_THRESHOLD_PERMILLE: u16 = 850;
const MAX_STARVATION_DEATH_RATE_PER_THOUSAND_YEAR: u32 = 750;
const MAX_CAPACITY_DEATH_RATE_PER_THOUSAND_YEAR: u32 = 300;
const BIRTH_SALT: u64 = 0x4249_5254_485f_0001;
const BASE_DEATH_SALT: u64 = 0x4445_4154_485f_0001;
const STARVATION_SALT: u64 = 0x5354_4152_5645_0001;
const CAPACITY_SALT: u64 = 0x4341_5041_4349_0001;

/// Deterministic population totals over authoritative HumanGroup runtime state.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HumanGroupPopulationAggregate {
    pub group_count: usize,
    pub total_population: u64,
    pub by_region: BTreeMap<RegionId, u64>,
}

/// One daily Stage 3.2 population/survival update summary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PopulationSurvivalReport {
    pub groups_updated: u64,
    pub total_population_before: u64,
    pub total_population_after: u64,
    pub births: u64,
    pub deaths: u64,
    pub starvation_deaths: u64,
    pub capacity_pressure_deaths: u64,
    pub overloaded_groups: u64,
}

/// Aggregates live HumanGroup population by world and Region without relying on
/// a fixed group count or a fixed Region count.
#[must_use]
pub fn aggregate_human_group_population(world: &WorldState) -> HumanGroupPopulationAggregate {
    let mut aggregate = HumanGroupPopulationAggregate::default();
    for group in world.entities.human_groups.iter() {
        let Some(state) = group.state.as_ref() else {
            continue;
        };
        aggregate.group_count = aggregate.group_count.saturating_add(1);
        aggregate.total_population = aggregate.total_population.saturating_add(state.population);
        let region_population = aggregate.by_region.entry(state.region_id).or_insert(0);
        *region_population = region_population.saturating_add(state.population);
    }
    aggregate
}

/// Advances birth, mortality, malnutrition and carrying-capacity pressure for
/// every authoritative HumanGroup exactly once for the current simulation day.
///
/// Stage 3.3 remains responsible for gathering and food consumption. This
/// function consumes the resulting `nutrition_permille` state but does not
/// manufacture food or alter stocks.
pub(crate) fn advance_population_survival(world: &mut WorldState) -> PopulationSurvivalReport {
    let before = aggregate_human_group_population(world);
    if before.group_count == 0 {
        return PopulationSurvivalReport::default();
    }

    let terrain_effects = world.terrain.as_ref().map(derive_terrain_effects);
    let next_id = world.entities.human_groups.next_id();
    let mut records = world
        .entities
        .human_groups
        .iter()
        .cloned()
        .collect::<Vec<_>>();
    let mut report = PopulationSurvivalReport {
        total_population_before: before.total_population,
        ..PopulationSurvivalReport::default()
    };

    for group in &mut records {
        let Some(state) = group.state.as_mut() else {
            continue;
        };
        report.groups_updated = report.groups_updated.saturating_add(1);

        let region_population = before
            .by_region
            .get(&state.region_id)
            .copied()
            .unwrap_or(0);
        let carrying_capacity = terrain_effects.as_ref().and_then(|effects| {
            carrying_capacity_for_region(
                world,
                effects,
                state.region_id,
                state.terrain_sample_index,
            )
        });
        let capacity_birth_factor = carrying_capacity
            .map(|capacity| capacity_birth_factor_permille(region_population, capacity))
            .unwrap_or(1_000);
        let capacity_death_rate = carrying_capacity
            .map(|capacity| capacity_death_rate_per_thousand(region_population, capacity))
            .unwrap_or(0);
        if capacity_death_rate > 0 {
            report.overloaded_groups = report.overloaded_groups.saturating_add(1);
        }

        let birth_rate = birth_rate_per_thousand(state.nutrition_permille, capacity_birth_factor);
        let base_death_rate = base_death_rate_per_thousand(state.risk_permille);
        let starvation_rate = starvation_death_rate_per_thousand(state.nutrition_permille);

        let starting_population = state.population;
        let births = deterministic_rate_count(
            starting_population,
            birth_rate,
            world.seed,
            state.id,
            world.elapsed_days,
            BIRTH_SALT,
        );
        let population_with_births = starting_population.saturating_add(births);
        let maximum_deaths = population_with_births.saturating_sub(1);

        let requested_base_deaths = deterministic_rate_count(
            starting_population,
            base_death_rate,
            world.seed,
            state.id,
            world.elapsed_days,
            BASE_DEATH_SALT,
        );
        let base_deaths = requested_base_deaths.min(maximum_deaths);
        let remaining_after_base = maximum_deaths.saturating_sub(base_deaths);

        let requested_starvation_deaths = deterministic_rate_count(
            starting_population,
            starvation_rate,
            world.seed,
            state.id,
            world.elapsed_days,
            STARVATION_SALT,
        );
        let starvation_deaths = requested_starvation_deaths.min(remaining_after_base);
        let remaining_after_starvation = remaining_after_base.saturating_sub(starvation_deaths);

        let requested_capacity_deaths = deterministic_rate_count(
            starting_population,
            capacity_death_rate,
            world.seed,
            state.id,
            world.elapsed_days,
            CAPACITY_SALT,
        );
        let capacity_deaths = requested_capacity_deaths.min(remaining_after_starvation);
        let deaths = base_deaths
            .saturating_add(starvation_deaths)
            .saturating_add(capacity_deaths);

        state.population = population_with_births.saturating_sub(deaths).max(1);
        report.births = report.births.saturating_add(births);
        report.deaths = report.deaths.saturating_add(deaths);
        report.starvation_deaths = report.starvation_deaths.saturating_add(starvation_deaths);
        report.capacity_pressure_deaths = report
            .capacity_pressure_deaths
            .saturating_add(capacity_deaths);
    }

    world.entities.human_groups = HumanGroupRegistry::from_parts(next_id, records)
        .expect("Stage 3.2 population update preserves HumanGroup registry invariants");
    report.total_population_after = aggregate_human_group_population(world).total_population;
    report
}

fn birth_rate_per_thousand(nutrition_permille: u16, capacity_factor_permille: u16) -> u32 {
    BASE_BIRTH_RATE_PER_THOUSAND_YEAR
        .saturating_mul(u32::from(nutrition_permille))
        .saturating_mul(u32::from(capacity_factor_permille))
        / 1_000_000
}

fn base_death_rate_per_thousand(risk_permille: u16) -> u32 {
    BASE_DEATH_RATE_PER_THOUSAND_YEAR.saturating_add(
        u32::from(risk_permille).saturating_mul(MAX_RISK_DEATH_RATE_PER_THOUSAND_YEAR) / 1_000,
    )
}

fn starvation_death_rate_per_thousand(nutrition_permille: u16) -> u32 {
    if nutrition_permille >= MALNUTRITION_THRESHOLD_PERMILLE {
        return 0;
    }
    let deficit = u32::from(MALNUTRITION_THRESHOLD_PERMILLE - nutrition_permille);
    deficit.saturating_mul(MAX_STARVATION_DEATH_RATE_PER_THOUSAND_YEAR)
        / u32::from(MALNUTRITION_THRESHOLD_PERMILLE)
}

fn capacity_birth_factor_permille(region_population: u64, capacity: u64) -> u16 {
    if region_population == 0 {
        return 1_000;
    }
    if capacity == 0 {
        return 0;
    }
    let factor = u128::from(capacity)
        .saturating_mul(1_000)
        .checked_div(u128::from(region_population))
        .unwrap_or(0)
        .min(1_000);
    u16::try_from(factor).unwrap_or(1_000)
}

fn capacity_death_rate_per_thousand(region_population: u64, capacity: u64) -> u32 {
    if region_population <= capacity {
        return 0;
    }
    if capacity == 0 {
        return MAX_CAPACITY_DEATH_RATE_PER_THOUSAND_YEAR;
    }
    let excess = region_population.saturating_sub(capacity);
    let overload_permille = u128::from(excess)
        .saturating_mul(1_000)
        .checked_div(u128::from(capacity))
        .unwrap_or(u128::MAX)
        .min(1_000);
    u32::try_from(overload_permille)
        .unwrap_or(1_000)
        .saturating_mul(MAX_CAPACITY_DEATH_RATE_PER_THOUSAND_YEAR)
        / 1_000
}

fn carrying_capacity_for_region(
    world: &WorldState,
    effects: &simulation_model::TerrainEffectField,
    region_id: RegionId,
    fallback_sample_index: u32,
) -> Option<u64> {
    let terrain = world.terrain.as_ref()?;
    let region = world.spatial.region(region_id)?;
    let sample_index = sample_index_for_point(terrain, region.center.x_m, region.center.z_m)
        .or_else(|| usize::try_from(fallback_sample_index).ok())?;
    let carrying_per_km2 = u128::from(
        effects
            .sample(sample_index)?
            .carrying_capacity_people_per_km2,
    );
    let area_m2 = polygon_area_m2(&region.boundary)?;
    let capacity = carrying_per_km2.saturating_mul(area_m2) / 1_000_000;
    Some(u64::try_from(capacity).unwrap_or(u64::MAX))
}

fn sample_index_for_point(terrain: &TerrainState, x_m: i32, z_m: i32) -> Option<usize> {
    if terrain.spacing_m <= 0 {
        return None;
    }
    let x_offset = i64::from(x_m) - i64::from(terrain.bounds.min_x_m);
    let z_offset = i64::from(z_m) - i64::from(terrain.bounds.min_z_m);
    if x_offset < 0 || z_offset < 0 {
        return None;
    }
    let spacing = i64::from(terrain.spacing_m);
    let x = x_offset / spacing;
    let z = z_offset / spacing;
    let width = i64::from(terrain.width);
    let height = i64::from(terrain.height);
    if x >= width || z >= height {
        return None;
    }
    usize::try_from(z.saturating_mul(width).saturating_add(x)).ok()
}

fn polygon_area_m2(boundary: &[simulation_model::MapPoint]) -> Option<u128> {
    if boundary.len() < 3 {
        return None;
    }
    let mut twice_area = 0_i128;
    for index in 0..boundary.len() {
        let current = boundary[index];
        let next = boundary[(index + 1) % boundary.len()];
        twice_area = twice_area
            .saturating_add(i128::from(current.x_m).saturating_mul(i128::from(next.z_m)))
            .saturating_sub(i128::from(next.x_m).saturating_mul(i128::from(current.z_m)));
    }
    Some(twice_area.unsigned_abs() / 2)
}

fn deterministic_rate_count(
    population: u64,
    annual_rate_per_thousand: u32,
    world_seed: u64,
    group_id: HumanGroupId,
    elapsed_days: u64,
    salt: u64,
) -> u64 {
    if population == 0 || annual_rate_per_thousand == 0 {
        return 0;
    }
    let numerator = u128::from(population).saturating_mul(u128::from(annual_rate_per_thousand));
    let whole = numerator / RATE_DENOMINATOR_PER_DAY;
    let remainder = numerator % RATE_DENOMINATOR_PER_DAY;
    let draw = u128::from(mix64(
        world_seed ^ group_id.0.rotate_left(17) ^ elapsed_days.rotate_left(31) ^ salt,
    )) % RATE_DENOMINATOR_PER_DAY;
    let count = whole.saturating_add(u128::from(draw < remainder));
    u64::try_from(count).unwrap_or(u64::MAX)
}

fn mix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

#[cfg(test)]
mod tests {
    use simulation_model::{
        BiomeClass, HumanGroupBehaviorProfile, HumanGroupInitialState, HumanGroupRuntimeSeed,
        InitialKnowledgeProfile, MapPoint, RegionId, RegionPoliticalState, RegionState,
        RegionSurface, ReliefClass, TerrainSample, TerrainState, WorldBounds, WorldSpatialState,
        WorldState, TERRAIN_SAMPLE_COUNT,
    };

    use super::{advance_population_survival, aggregate_human_group_population};

    fn world_with_group(population: u64, nutrition: u16, region_half_extent_m: i32) -> WorldState {
        let terrain = TerrainState::new_trial(vec![
            TerrainSample {
                elevation_m: 100,
                moisture_permille: 550,
                relief: ReliefClass::Plains,
                biome: BiomeClass::Grassland,
            };
            TERRAIN_SAMPLE_COUNT
        ])
        .expect("terrain");
        let region = RegionState {
            id: RegionId(1),
            surface: RegionSurface::Land,
            center: MapPoint::new(0, 0),
            boundary: vec![
                MapPoint::new(-region_half_extent_m, -region_half_extent_m),
                MapPoint::new(region_half_extent_m, -region_half_extent_m),
                MapPoint::new(region_half_extent_m, region_half_extent_m),
                MapPoint::new(-region_half_extent_m, region_half_extent_m),
            ],
            neighbors: Vec::new(),
            political: RegionPoliticalState::unclaimed(),
        };
        let mut world = WorldState::new(2026);
        world.terrain = Some(terrain);
        world.spatial = WorldSpatialState::new(
            WorldBounds::new(-640_000, 640_000, -640_000, 640_000).expect("bounds"),
            vec![region],
        )
        .expect("spatial");
        world
            .entities
            .human_groups
            .create_initialized_with_runtime(
                HumanGroupInitialState {
                    region_id: RegionId(1),
                    x_m: 0,
                    z_m: 0,
                    terrain_sample_index: 8_320,
                    population,
                    food_stock_person_days: population.saturating_mul(30),
                    basic_resource_stock_units: population,
                    behavior: HumanGroupBehaviorProfile {
                        mobility_permille: 300,
                        exploration_permille: 400,
                        settlement_bias_permille: 300,
                    },
                    knowledge: InitialKnowledgeProfile::default(),
                },
                HumanGroupRuntimeSeed {
                    nutrition_permille: nutrition,
                    cohesion_permille: 500,
                    risk_permille: 500,
                },
            )
            .expect("group");
        world
    }

    #[test]
    fn aggregate_population_is_dynamic_and_region_based() {
        let world = world_with_group(12_345, 1_000, 100_000);
        let aggregate = aggregate_human_group_population(&world);
        assert_eq!(aggregate.group_count, 1);
        assert_eq!(aggregate.total_population, 12_345);
        assert_eq!(aggregate.by_region.get(&RegionId(1)), Some(&12_345));
    }

    #[test]
    fn same_seed_and_day_produce_identical_demography() {
        let mut first = world_with_group(1_000_000, 1_000, 200_000);
        let mut second = first.clone();
        let first_report = advance_population_survival(&mut first);
        let second_report = advance_population_survival(&mut second);
        assert_eq!(first_report, second_report);
        assert_eq!(first, second);
    }

    #[test]
    fn malnutrition_adds_starvation_mortality() {
        let mut nourished = world_with_group(1_000_000, 1_000, 200_000);
        let mut malnourished = world_with_group(1_000_000, 200, 200_000);
        let healthy_report = advance_population_survival(&mut nourished);
        let hungry_report = advance_population_survival(&mut malnourished);
        assert_eq!(healthy_report.starvation_deaths, 0);
        assert!(hungry_report.starvation_deaths > 0);
        assert!(hungry_report.deaths > healthy_report.deaths);
    }

    #[test]
    fn carrying_capacity_pressure_suppresses_births_and_adds_deaths() {
        let mut roomy = world_with_group(1_000_000, 1_000, 200_000);
        let mut crowded = world_with_group(1_000_000, 1_000, 500);
        let roomy_report = advance_population_survival(&mut roomy);
        let crowded_report = advance_population_survival(&mut crowded);
        assert_eq!(roomy_report.overloaded_groups, 0);
        assert!(crowded_report.overloaded_groups > 0);
        assert!(crowded_report.capacity_pressure_deaths > 0);
        assert!(crowded_report.births < roomy_report.births);
    }

    #[test]
    fn survival_update_never_creates_zero_population() {
        let mut world = world_with_group(1, 0, 1);
        for day in 0..10_000 {
            world.elapsed_days = day;
            advance_population_survival(&mut world);
            let population = aggregate_human_group_population(&world).total_population;
            assert_eq!(population, 1);
        }
    }
}
