use simulation_model::{
    EntityRegistry, HumanGroupBehaviorProfile, HumanGroupId, HumanGroupInitialState,
    HumanGroupRegistry, HumanGroupRuntimeSeed, InitialKnowledgeProfile, RegionId, RegionSurface,
    ResourceFieldState, TerrainEffectField, TerrainState, WorldSpatialState,
};

const HABITAT_SALT: u64 = 0x4841_4249_5441_5401;
const POPULATION_SALT: u64 = 0x504f_5055_4c41_5401;
const MOBILITY_SALT: u64 = 0x4d4f_4249_4c49_5401;
const EXPLORATION_SALT: u64 = 0x4558_504c_4f52_4501;
const SETTLEMENT_SALT: u64 = 0x5345_5454_4c45_0101;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PermilleRange {
    pub min: u16,
    pub max: u16,
}

impl PermilleRange {
    #[must_use]
    pub const fn is_valid(self) -> bool {
        self.min <= self.max && self.max <= 1_000
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitialHumanGroupConfig {
    pub group_count: usize,
    pub total_population: u64,
    pub population_variation_permille: u16,
    pub initial_food_days_min: u16,
    pub initial_food_days_max: u16,
    pub basic_resource_units_per_person: u16,
    pub mobility_permille: PermilleRange,
    pub exploration_permille: PermilleRange,
    pub settlement_bias_permille: PermilleRange,
    pub runtime_seed: HumanGroupRuntimeSeed,
    pub contact_radius_m: u32,
    pub knowledge_profile: InitialKnowledgeProfile,
}

impl InitialHumanGroupConfig {
    /// Validates an explicit civilization-origin initialization configuration.
    ///
    /// No default group count or population is supplied here: those values are
    /// scenario parameters rather than simulation constants.
    ///
    /// # Errors
    /// Returns [`HumanGroupGenerationError::InvalidConfig`] when a parameter is
    /// internally inconsistent.
    pub fn validate(&self) -> Result<(), HumanGroupGenerationError> {
        if self.group_count == 0
            || self.total_population < u64::try_from(self.group_count).unwrap_or(u64::MAX)
            || self.population_variation_permille > 1_000
            || self.initial_food_days_min == 0
            || self.initial_food_days_min > self.initial_food_days_max
            || !self.mobility_permille.is_valid()
            || !self.exploration_permille.is_valid()
            || !self.settlement_bias_permille.is_valid()
            || !self.runtime_seed.is_valid()
            || self.contact_radius_m == 0
            || !self.knowledge_profile.is_valid()
        {
            return Err(HumanGroupGenerationError::InvalidConfig);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InitialGroupContact {
    pub first: HumanGroupId,
    pub second: HumanGroupId,
    pub distance_m: u64,
    pub contact_possible: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InitialHumanGroupReport {
    pub group_count: usize,
    pub total_population: u64,
    pub habitable_region_count: usize,
    pub contact_pair_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HumanGroupGenerationError {
    InvalidConfig,
    MissingTerrainAlignment,
    MissingResourceAlignment,
    NoHabitableRegion,
    InsufficientHabitableRegions,
    PopulationOverflow,
    RegistryFailure,
}

impl core::fmt::Display for HumanGroupGenerationError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(match self {
            Self::InvalidConfig => "initial HumanGroup configuration is invalid",
            Self::MissingTerrainAlignment => "Region centers do not align with canonical terrain",
            Self::MissingResourceAlignment => "terrain/resource/effect fields are not aligned",
            Self::NoHabitableRegion => "no habitable land Region is available",
            Self::InsufficientHabitableRegions => {
                "requested HumanGroup count exceeds distinct habitable Regions"
            }
            Self::PopulationOverflow => "initial HumanGroup population arithmetic overflowed",
            Self::RegistryFailure => "initial HumanGroup registry construction failed",
        })
    }
}

impl std::error::Error for HumanGroupGenerationError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct HabitatSample {
    region_id: RegionId,
    x_m: i32,
    z_m: i32,
    sample_index: usize,
    food_capacity: u32,
    carrying_capacity: u16,
    construction_accessibility: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct HabitatCandidate {
    region_id: RegionId,
    x_m: i32,
    z_m: i32,
    sample_index: usize,
    construction_accessibility: u16,
    selection_score: u64,
    natural_food_access_permille: u16,
}

/// Generates the Year-1 pre-state `HumanGroup` registry from explicit parameters
/// and existing geography. Countries, borders, GDP and fiscal state are not
/// created by this function.
///
/// Each initial group occupies a distinct habitable Region. Habitat ranking is
/// based on the Region-center terrain sample's natural food base and carrying
/// capacity plus a deterministic seed-derived tie-breaker. No country role or
/// future civilization outcome participates.
///
/// # Errors
/// Returns [`HumanGroupGenerationError`] for invalid parameters, missing field
/// alignment or insufficient habitable Regions.
pub fn generate_initial_human_groups(
    seed: u64,
    config: &InitialHumanGroupConfig,
    spatial: &WorldSpatialState,
    terrain: &TerrainState,
    resources: &ResourceFieldState,
    effects: &TerrainEffectField,
) -> Result<HumanGroupRegistry, HumanGroupGenerationError> {
    config.validate()?;
    validate_alignment(terrain, resources, effects)?;
    let samples = collect_habitat_samples(spatial, terrain, resources, effects)?;
    if samples.len() < config.group_count {
        return Err(HumanGroupGenerationError::InsufficientHabitableRegions);
    }
    let mut candidates = rank_habitat_candidates(seed, samples);
    candidates.truncate(config.group_count);
    let populations = distribute_population(seed, config)?;
    build_registry(seed, config, &candidates, populations)
}

fn validate_alignment(
    terrain: &TerrainState,
    resources: &ResourceFieldState,
    effects: &TerrainEffectField,
) -> Result<(), HumanGroupGenerationError> {
    if terrain.samples.len() != resources.cells.len()
        || terrain.samples.len() != effects.samples.len()
    {
        return Err(HumanGroupGenerationError::MissingResourceAlignment);
    }
    Ok(())
}

fn collect_habitat_samples(
    spatial: &WorldSpatialState,
    terrain: &TerrainState,
    resources: &ResourceFieldState,
    effects: &TerrainEffectField,
) -> Result<Vec<HabitatSample>, HumanGroupGenerationError> {
    let mut samples = Vec::new();
    for region in &spatial.regions {
        if region.surface != RegionSurface::Land {
            continue;
        }
        let sample_index = sample_index_for_point(terrain, region.center.x_m, region.center.z_m)
            .ok_or(HumanGroupGenerationError::MissingTerrainAlignment)?;
        let terrain_sample = terrain
            .samples
            .get(sample_index)
            .ok_or(HumanGroupGenerationError::MissingTerrainAlignment)?;
        let effect = effects
            .sample(sample_index)
            .ok_or(HumanGroupGenerationError::MissingResourceAlignment)?;
        let resource = resources
            .cell(sample_index)
            .ok_or(HumanGroupGenerationError::MissingResourceAlignment)?;
        if terrain_sample.elevation_m < terrain.sea_level_m
            || effect.carrying_capacity_people_per_km2 == 0
            || resource.food_capacity_tonnes_per_year == 0
        {
            continue;
        }
        samples.push(HabitatSample {
            region_id: region.id,
            x_m: region.center.x_m,
            z_m: region.center.z_m,
            sample_index,
            food_capacity: resource.food_capacity_tonnes_per_year,
            carrying_capacity: effect.carrying_capacity_people_per_km2,
            construction_accessibility: resource.construction.accessibility_permille,
        });
    }
    if samples.is_empty() {
        return Err(HumanGroupGenerationError::NoHabitableRegion);
    }
    Ok(samples)
}

fn rank_habitat_candidates(seed: u64, samples: Vec<HabitatSample>) -> Vec<HabitatCandidate> {
    let max_food = samples
        .iter()
        .map(|sample| sample.food_capacity)
        .max()
        .unwrap_or(1)
        .max(1);
    let max_carrying = samples
        .iter()
        .map(|sample| sample.carrying_capacity)
        .max()
        .unwrap_or(1)
        .max(1);
    let mut candidates = samples
        .into_iter()
        .map(|sample| candidate_from_sample(seed, sample, max_food, max_carrying))
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        right
            .selection_score
            .cmp(&left.selection_score)
            .then_with(|| left.region_id.cmp(&right.region_id))
    });
    candidates
}

fn candidate_from_sample(
    seed: u64,
    sample: HabitatSample,
    max_food: u32,
    max_carrying: u16,
) -> HabitatCandidate {
    let food_permille =
        u16::try_from(u64::from(sample.food_capacity).saturating_mul(1_000) / u64::from(max_food))
            .unwrap_or(1_000);
    let carrying_permille = u16::try_from(
        u64::from(sample.carrying_capacity).saturating_mul(1_000) / u64::from(max_carrying),
    )
    .unwrap_or(1_000);
    let natural_food_access_permille =
        u16::try_from((u32::from(food_permille) + u32::from(carrying_permille)) / 2)
            .unwrap_or(1_000);
    let habitat_score = u64::from(food_permille)
        .saturating_mul(3)
        .saturating_add(u64::from(carrying_permille).saturating_mul(2));
    let jitter = mix64(seed ^ sample.region_id.0 ^ HABITAT_SALT) & 0x3ff;
    HabitatCandidate {
        region_id: sample.region_id,
        x_m: sample.x_m,
        z_m: sample.z_m,
        sample_index: sample.sample_index,
        construction_accessibility: sample.construction_accessibility,
        selection_score: habitat_score.saturating_mul(1_024).saturating_add(jitter),
        natural_food_access_permille,
    }
}

fn build_registry(
    seed: u64,
    config: &InitialHumanGroupConfig,
    candidates: &[HabitatCandidate],
    populations: Vec<u64>,
) -> Result<HumanGroupRegistry, HumanGroupGenerationError> {
    let mut registry = HumanGroupRegistry::new();
    for (ordinal, (candidate, population)) in candidates.iter().zip(populations).enumerate() {
        let ordinal_u64 = u64::try_from(ordinal).unwrap_or(u64::MAX);
        let initial = build_initial_state(seed, config, *candidate, population, ordinal_u64)?;
        registry
            .create_initialized_with_runtime(initial, config.runtime_seed)
            .map_err(|_| HumanGroupGenerationError::RegistryFailure)?;
    }
    Ok(registry)
}

fn build_initial_state(
    seed: u64,
    config: &InitialHumanGroupConfig,
    candidate: HabitatCandidate,
    population: u64,
    ordinal: u64,
) -> Result<HumanGroupInitialState, HumanGroupGenerationError> {
    let food_days_span = u64::from(
        config
            .initial_food_days_max
            .saturating_sub(config.initial_food_days_min),
    );
    let food_days = u64::from(config.initial_food_days_min)
        + food_days_span.saturating_mul(u64::from(candidate.natural_food_access_permille)) / 1_000;
    let food_stock_person_days = population
        .checked_mul(food_days)
        .ok_or(HumanGroupGenerationError::PopulationOverflow)?;
    let basic_resource_stock_units = population
        .checked_mul(u64::from(config.basic_resource_units_per_person))
        .and_then(|value| {
            value
                .checked_mul(u64::from(candidate.construction_accessibility))
                .map(|scaled| scaled / 1_000)
        })
        .ok_or(HumanGroupGenerationError::PopulationOverflow)?;

    Ok(HumanGroupInitialState {
        region_id: candidate.region_id,
        x_m: candidate.x_m,
        z_m: candidate.z_m,
        terrain_sample_index: u32::try_from(candidate.sample_index)
            .map_err(|_| HumanGroupGenerationError::MissingTerrainAlignment)?,
        population,
        food_stock_person_days,
        basic_resource_stock_units,
        behavior: HumanGroupBehaviorProfile {
            mobility_permille: sample_permille_range(
                seed,
                ordinal,
                MOBILITY_SALT,
                config.mobility_permille,
            ),
            exploration_permille: sample_permille_range(
                seed,
                ordinal,
                EXPLORATION_SALT,
                config.exploration_permille,
            ),
            settlement_bias_permille: sample_permille_range(
                seed,
                ordinal,
                SETTLEMENT_SALT,
                config.settlement_bias_permille,
            ),
        },
        knowledge: config.knowledge_profile.clone(),
    })
}

#[must_use]
pub fn initial_group_contacts(
    registry: &HumanGroupRegistry,
    contact_radius_m: u32,
) -> Vec<InitialGroupContact> {
    let groups = registry
        .iter()
        .filter_map(|group| group.initial.as_ref().map(|initial| (group.id, initial)))
        .collect::<Vec<_>>();
    let mut contacts = Vec::new();
    for first_index in 0..groups.len() {
        for second_index in (first_index + 1)..groups.len() {
            let (first_id, first) = groups[first_index];
            let (second_id, second) = groups[second_index];
            let distance_m = integer_distance_m(first.x_m, first.z_m, second.x_m, second.z_m);
            contacts.push(InitialGroupContact {
                first: first_id,
                second: second_id,
                distance_m,
                contact_possible: distance_m <= u64::from(contact_radius_m),
            });
        }
    }
    contacts
}

#[must_use]
pub fn initial_human_group_report(
    registry: &HumanGroupRegistry,
    habitable_region_count: usize,
    contact_radius_m: u32,
) -> InitialHumanGroupReport {
    InitialHumanGroupReport {
        group_count: registry.len(),
        total_population: registry
            .iter()
            .filter_map(|group| group.initial.as_ref())
            .map(|initial| initial.population)
            .sum(),
        habitable_region_count,
        contact_pair_count: initial_group_contacts(registry, contact_radius_m).len(),
    }
}

fn distribute_population(
    seed: u64,
    config: &InitialHumanGroupConfig,
) -> Result<Vec<u64>, HumanGroupGenerationError> {
    let count = u64::try_from(config.group_count)
        .map_err(|_| HumanGroupGenerationError::PopulationOverflow)?;
    let variation = u64::from(config.population_variation_permille);
    let mut weights = Vec::with_capacity(config.group_count);
    for ordinal in 0..config.group_count {
        let ordinal_u64 = u64::try_from(ordinal).unwrap_or(u64::MAX);
        let span = variation.saturating_mul(2).saturating_add(1);
        let jitter = mix64(seed ^ POPULATION_SALT ^ ordinal_u64) % span;
        let weight = 1_000_u64
            .saturating_sub(variation)
            .saturating_add(jitter)
            .max(1);
        weights.push(weight);
    }
    let weight_sum = weights.iter().copied().sum::<u64>().max(1);
    let distributable = config.total_population.saturating_sub(count);
    let mut populations = vec![1_u64; config.group_count];
    let mut assigned = 0_u64;
    for (population, weight) in populations.iter_mut().zip(&weights) {
        let share =
            u128::from(distributable).saturating_mul(u128::from(*weight)) / u128::from(weight_sum);
        let share =
            u64::try_from(share).map_err(|_| HumanGroupGenerationError::PopulationOverflow)?;
        *population = population
            .checked_add(share)
            .ok_or(HumanGroupGenerationError::PopulationOverflow)?;
        assigned = assigned
            .checked_add(share)
            .ok_or(HumanGroupGenerationError::PopulationOverflow)?;
    }
    distribute_population_remainder(
        seed,
        config.group_count,
        distributable,
        assigned,
        &mut populations,
    )?;
    Ok(populations)
}

fn distribute_population_remainder(
    seed: u64,
    group_count: usize,
    distributable: u64,
    assigned: u64,
    populations: &mut [u64],
) -> Result<(), HumanGroupGenerationError> {
    let count =
        u64::try_from(group_count).map_err(|_| HumanGroupGenerationError::PopulationOverflow)?;
    let mut remainder = distributable.saturating_sub(assigned);
    let rotation = usize::try_from(mix64(seed ^ POPULATION_SALT) % count).unwrap_or(0);
    let mut cursor = 0_usize;
    while remainder > 0 {
        let index = (rotation + cursor) % group_count;
        populations[index] = populations[index]
            .checked_add(1)
            .ok_or(HumanGroupGenerationError::PopulationOverflow)?;
        remainder -= 1;
        cursor += 1;
    }
    Ok(())
}

fn sample_permille_range(seed: u64, ordinal: u64, salt: u64, range: PermilleRange) -> u16 {
    let span = u64::from(range.max - range.min) + 1;
    let offset = mix64(seed ^ ordinal.wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ salt) % span;
    range.min.saturating_add(u16::try_from(offset).unwrap_or(0))
}

fn sample_index_for_point(terrain: &TerrainState, x_m: i32, z_m: i32) -> Option<usize> {
    if x_m < terrain.bounds.min_x_m
        || x_m > terrain.bounds.max_x_m
        || z_m < terrain.bounds.min_z_m
        || z_m > terrain.bounds.max_z_m
        || terrain.spacing_m <= 0
    {
        return None;
    }
    let spacing = i64::from(terrain.spacing_m);
    let x_offset = i64::from(x_m) - i64::from(terrain.bounds.min_x_m);
    let z_offset = i64::from(z_m) - i64::from(terrain.bounds.min_z_m);
    let x =
        ((x_offset + spacing / 2) / spacing).clamp(0, i64::from(terrain.width.saturating_sub(1)));
    let z =
        ((z_offset + spacing / 2) / spacing).clamp(0, i64::from(terrain.height.saturating_sub(1)));
    let x = usize::try_from(x).ok()?;
    let z = usize::try_from(z).ok()?;
    z.checked_mul(usize::from(terrain.width))?.checked_add(x)
}

fn integer_distance_m(first_x: i32, first_z: i32, second_x: i32, second_z: i32) -> u64 {
    let dx = i128::from(first_x) - i128::from(second_x);
    let dz = i128::from(first_z) - i128::from(second_z);
    let squared = u128::try_from(dx * dx + dz * dz).unwrap_or(u128::MAX);
    integer_sqrt(squared)
}

fn integer_sqrt(value: u128) -> u64 {
    if value == 0 {
        return 0;
    }
    let mut low = 1_u128;
    let mut high = value.min(u128::from(u64::MAX));
    while low <= high {
        let mid = low + (high - low) / 2;
        let quotient = value / mid;
        if mid <= quotient {
            low = mid + 1;
        } else {
            high = mid - 1;
        }
    }
    u64::try_from(high).unwrap_or(u64::MAX)
}

fn mix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}
