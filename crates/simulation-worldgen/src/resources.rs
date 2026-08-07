use simulation_model::{
    derive_terrain_effects, food_capacity_from_terrain_effects, BiomeClass, ReliefClass,
    ResourceCellState, ResourceDeposit, ResourceError, ResourceFieldState, TerrainEffectField,
    TerrainState, TERRAIN_SAMPLE_COUNT,
};

const FOOD_FERTILITY_SALT: u64 = 0x464f_4f44_4645_5254;
const ENERGY_GEOLOGY_SALT: u64 = 0x454e_4552_4759_4745;
const ENERGY_QUALITY_SALT: u64 = 0x454e_4552_4759_5155;
const METAL_GEOLOGY_SALT: u64 = 0x4d45_5441_4c47_454f;
const METAL_QUALITY_SALT: u64 = 0x4d45_5441_4c51_5541;
const CONSTRUCTION_GEOLOGY_SALT: u64 = 0x434f_4e53_5447_454f;
const CONSTRUCTION_QUALITY_SALT: u64 = 0x434f_4e53_5451_5541;
const ACCESSIBILITY_SALT: u64 = 0x4143_4345_5353_4942;

/// Distribution diagnostics used to ensure a seed produces useful scarcity
/// without collapsing a resource family into one or two cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceBalanceReport {
    pub land_cells: usize,
    pub food_cells: usize,
    pub energy_cells: usize,
    pub metal_cells: usize,
    pub construction_cells: usize,
    pub total_food_capacity_tonnes_per_year: u64,
    pub total_energy_quantity: u64,
    pub total_metal_quantity: u64,
    pub total_construction_quantity: u64,
    pub energy_top_decile_share_permille: u16,
    pub metal_top_decile_share_permille: u16,
    pub construction_top_decile_share_permille: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceGenerationError {
    ResourceState(ResourceError),
    TerrainEffectAlignment,
    NoLand,
    MissingFoodBase,
    EnergyCoverageOutOfRange { permille: u16 },
    MetalCoverageOutOfRange { permille: u16 },
    ConstructionCoverageOutOfRange { permille: u16 },
    ExcessiveConcentration { resource: &'static str, permille: u16 },
}

impl core::fmt::Display for ResourceGenerationError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ResourceState(error) => write!(formatter, "resource state invalid: {error}"),
            Self::TerrainEffectAlignment => {
                formatter.write_str("terrain effect field does not align with terrain samples")
            }
            Self::NoLand => formatter.write_str("resource generation requires at least one land cell"),
            Self::MissingFoodBase => formatter.write_str("generated land has no renewable food base"),
            Self::EnergyCoverageOutOfRange { permille } => {
                write!(formatter, "energy deposit coverage {permille} permille is outside TEST bounds")
            }
            Self::MetalCoverageOutOfRange { permille } => {
                write!(formatter, "metal deposit coverage {permille} permille is outside TEST bounds")
            }
            Self::ConstructionCoverageOutOfRange { permille } => write!(
                formatter,
                "construction resource coverage {permille} permille is outside TEST bounds"
            ),
            Self::ExcessiveConcentration { resource, permille } => write!(
                formatter,
                "{resource} top-decile concentration {permille} permille exceeds TEST bound"
            ),
        }
    }
}

impl std::error::Error for ResourceGenerationError {}

impl From<ResourceError> for ResourceGenerationError {
    fn from(error: ResourceError) -> Self {
        Self::ResourceState(error)
    }
}

/// Generates the canonical Stage 2.4 resource substrate from seed and terrain.
///
/// Food is a renewable annual geography-only capacity. Energy, metals and
/// construction resources are finite authoritative deposits. All calculations
/// use integer arithmetic so the same seed and terrain are byte reproducible.
///
/// # Errors
///
/// Returns [`ResourceGenerationError`] if the terrain/effect fields disagree,
/// generated resource state violates invariants, or distribution balance falls
/// outside the TEST acceptance envelope.
pub fn generate_trial_resources(
    seed: u64,
    terrain: &TerrainState,
) -> Result<ResourceFieldState, ResourceGenerationError> {
    let effects = derive_terrain_effects(terrain);
    if terrain.samples.len() != TERRAIN_SAMPLE_COUNT || effects.samples.len() != terrain.samples.len()
    {
        return Err(ResourceGenerationError::TerrainEffectAlignment);
    }

    let mut cells = Vec::with_capacity(terrain.samples.len());
    for (index, sample) in terrain.samples.iter().copied().enumerate() {
        if sample.elevation_m < terrain.sea_level_m {
            cells.push(ResourceCellState::OCEAN);
            continue;
        }

        let food_capacity_tonnes_per_year = food_capacity(seed, &effects, index)
            .ok_or(ResourceGenerationError::TerrainEffectAlignment)?;
        let energy = energy_deposit(seed, index, sample, &effects)?;
        let metals = metal_deposit(seed, index, sample, &effects)?;
        let construction = construction_deposit(seed, index, sample, &effects)?;
        cells.push(ResourceCellState {
            food_capacity_tonnes_per_year,
            energy,
            metals,
            construction,
        });
    }

    let field = ResourceFieldState::new(cells)?;
    field.validate_against_terrain(terrain)?;
    validate_resource_balance(&field, terrain)?;
    Ok(field)
}

/// Computes deterministic distribution diagnostics without mutating stock.
#[must_use]
pub fn resource_balance_report(
    field: &ResourceFieldState,
    terrain: &TerrainState,
) -> ResourceBalanceReport {
    let mut report = ResourceBalanceReport {
        land_cells: 0,
        food_cells: 0,
        energy_cells: 0,
        metal_cells: 0,
        construction_cells: 0,
        total_food_capacity_tonnes_per_year: 0,
        total_energy_quantity: 0,
        total_metal_quantity: 0,
        total_construction_quantity: 0,
        energy_top_decile_share_permille: 0,
        metal_top_decile_share_permille: 0,
        construction_top_decile_share_permille: 0,
    };
    let mut energy_quantities = Vec::new();
    let mut metal_quantities = Vec::new();
    let mut construction_quantities = Vec::new();

    for (terrain_sample, cell) in terrain.samples.iter().zip(&field.cells) {
        if terrain_sample.elevation_m < terrain.sea_level_m {
            continue;
        }
        report.land_cells += 1;
        if cell.food_capacity_tonnes_per_year > 0 {
            report.food_cells += 1;
        }
        report.total_food_capacity_tonnes_per_year = report
            .total_food_capacity_tonnes_per_year
            .saturating_add(u64::from(cell.food_capacity_tonnes_per_year));
        accumulate_deposit(
            cell.energy,
            &mut report.energy_cells,
            &mut report.total_energy_quantity,
            &mut energy_quantities,
        );
        accumulate_deposit(
            cell.metals,
            &mut report.metal_cells,
            &mut report.total_metal_quantity,
            &mut metal_quantities,
        );
        accumulate_deposit(
            cell.construction,
            &mut report.construction_cells,
            &mut report.total_construction_quantity,
            &mut construction_quantities,
        );
    }

    report.energy_top_decile_share_permille = top_decile_share_permille(&mut energy_quantities);
    report.metal_top_decile_share_permille = top_decile_share_permille(&mut metal_quantities);
    report.construction_top_decile_share_permille =
        top_decile_share_permille(&mut construction_quantities);
    report
}

/// Validates scarcity and concentration envelopes across the generated land.
/// These are TEST balance constraints, not empirical resource-distribution laws.
///
/// # Errors
///
/// Returns [`ResourceGenerationError`] if a resource family is absent, almost
/// ubiquitous, or pathologically concentrated for the trial-scale world.
pub fn validate_resource_balance(
    field: &ResourceFieldState,
    terrain: &TerrainState,
) -> Result<ResourceBalanceReport, ResourceGenerationError> {
    let report = resource_balance_report(field, terrain);
    if report.land_cells == 0 {
        return Err(ResourceGenerationError::NoLand);
    }
    if report.food_cells == 0 || report.total_food_capacity_tonnes_per_year == 0 {
        return Err(ResourceGenerationError::MissingFoodBase);
    }

    validate_coverage(
        "energy",
        report.energy_cells,
        report.land_cells,
        120,
        450,
    )?;
    validate_coverage(
        "metals",
        report.metal_cells,
        report.land_cells,
        100,
        420,
    )?;
    validate_coverage(
        "construction",
        report.construction_cells,
        report.land_cells,
        500,
        950,
    )?;

    for (resource, share) in [
        ("energy", report.energy_top_decile_share_permille),
        ("metals", report.metal_top_decile_share_permille),
        (
            "construction",
            report.construction_top_decile_share_permille,
        ),
    ] {
        if share > 650 {
            return Err(ResourceGenerationError::ExcessiveConcentration {
                resource,
                permille: share,
            });
        }
    }
    Ok(report)
}

fn food_capacity(seed: u64, effects: &TerrainEffectField, index: usize) -> Option<u32> {
    let base = u64::from(food_capacity_from_terrain_effects(effects, index)?);
    let fertility = 850_u64 + u64::from(random_permille(seed, index, FOOD_FERTILITY_SALT)) * 300 / 1_000;
    u32::try_from(base.saturating_mul(fertility) / 1_000).ok()
}

fn energy_deposit(
    seed: u64,
    index: usize,
    sample: simulation_model::TerrainSample,
    effects: &TerrainEffectField,
) -> Result<ResourceDeposit, ResourceGenerationError> {
    let geology = i32::from(random_permille(seed, index, ENERGY_GEOLOGY_SALT));
    let relief_adjustment = match sample.relief {
        ReliefClass::Coast => 90,
        ReliefClass::Plains => 70,
        ReliefClass::Hills => 20,
        ReliefClass::Mountains => -80,
        ReliefClass::DeepOcean | ReliefClass::ShallowOcean => -1_000,
    };
    let wetland_bonus = if sample.biome == BiomeClass::Wetland { 35 } else { 0 };
    let score = geology + relief_adjustment + wetland_bonus;
    if score < 790 {
        return Ok(ResourceDeposit::empty());
    }
    let richness = u64::try_from(score.saturating_sub(760)).unwrap_or(0);
    let quantity = 12_000_u64.saturating_add(richness.saturating_mul(650));
    deposit_with_quality_access(
        seed,
        index,
        quantity,
        ENERGY_QUALITY_SALT,
        effects,
    )
}

fn metal_deposit(
    seed: u64,
    index: usize,
    sample: simulation_model::TerrainSample,
    effects: &TerrainEffectField,
) -> Result<ResourceDeposit, ResourceGenerationError> {
    let geology = i32::from(random_permille(seed, index, METAL_GEOLOGY_SALT));
    let relief_adjustment = match sample.relief {
        ReliefClass::Mountains => 180,
        ReliefClass::Hills => 100,
        ReliefClass::Plains => -20,
        ReliefClass::Coast => -30,
        ReliefClass::DeepOcean | ReliefClass::ShallowOcean => -1_000,
    };
    let score = geology + relief_adjustment;
    if score < 810 {
        return Ok(ResourceDeposit::empty());
    }
    let richness = u64::try_from(score.saturating_sub(780)).unwrap_or(0);
    let quantity = 8_000_u64.saturating_add(richness.saturating_mul(420));
    deposit_with_quality_access(
        seed,
        index,
        quantity,
        METAL_QUALITY_SALT,
        effects,
    )
}

fn construction_deposit(
    seed: u64,
    index: usize,
    sample: simulation_model::TerrainSample,
    effects: &TerrainEffectField,
) -> Result<ResourceDeposit, ResourceGenerationError> {
    let geology = i32::from(random_permille(seed, index, CONSTRUCTION_GEOLOGY_SALT));
    let biome_adjustment = match sample.biome {
        BiomeClass::Forest => 260,
        BiomeClass::Grassland => 80,
        BiomeClass::Wetland => 20,
        BiomeClass::Desert => 50,
        BiomeClass::Alpine => 120,
        BiomeClass::Ocean => -1_000,
    };
    let relief_adjustment = match sample.relief {
        ReliefClass::Mountains => 160,
        ReliefClass::Hills => 120,
        ReliefClass::Plains => 20,
        ReliefClass::Coast => 10,
        ReliefClass::DeepOcean | ReliefClass::ShallowOcean => -1_000,
    };
    let score = geology + biome_adjustment + relief_adjustment;
    if score < 470 {
        return Ok(ResourceDeposit::empty());
    }
    let richness = u64::try_from(score.saturating_sub(430)).unwrap_or(0);
    let quantity = 15_000_u64.saturating_add(richness.saturating_mul(300));
    deposit_with_quality_access(
        seed,
        index,
        quantity,
        CONSTRUCTION_QUALITY_SALT,
        effects,
    )
}

fn deposit_with_quality_access(
    seed: u64,
    index: usize,
    quantity: u64,
    quality_salt: u64,
    effects: &TerrainEffectField,
) -> Result<ResourceDeposit, ResourceGenerationError> {
    let terrain_effect = effects
        .sample(index)
        .ok_or(ResourceGenerationError::TerrainEffectAlignment)?;
    let quality = 450_u16.saturating_add(random_permille(seed, index, quality_salt) * 500 / 1_000);
    let random_access = random_permille(seed, index, ACCESSIBILITY_SALT);
    let movement_penalty = terrain_effect.movement_cost_permille.saturating_sub(1_000) / 2;
    let construction_penalty = terrain_effect.construction_cost_permille.saturating_sub(1_000) / 4;
    let terrain_access = 1_000_u16
        .saturating_sub(movement_penalty)
        .saturating_sub(construction_penalty)
        .max(180);
    let accessibility = (u32::from(terrain_access) * 3 + u32::from(random_access)) / 4;
    ResourceDeposit::new(
        quantity,
        quality,
        u16::try_from(accessibility).unwrap_or(1_000),
    )
    .map_err(ResourceGenerationError::from)
}

fn validate_coverage(
    resource: &'static str,
    cells: usize,
    land_cells: usize,
    minimum_permille: u16,
    maximum_permille: u16,
) -> Result<(), ResourceGenerationError> {
    let coverage = ratio_permille(cells, land_cells);
    if coverage >= minimum_permille && coverage <= maximum_permille {
        return Ok(());
    }
    match resource {
        "energy" => Err(ResourceGenerationError::EnergyCoverageOutOfRange {
            permille: coverage,
        }),
        "metals" => Err(ResourceGenerationError::MetalCoverageOutOfRange {
            permille: coverage,
        }),
        _ => Err(ResourceGenerationError::ConstructionCoverageOutOfRange {
            permille: coverage,
        }),
    }
}

fn accumulate_deposit(
    deposit: ResourceDeposit,
    cell_count: &mut usize,
    total_quantity: &mut u64,
    quantities: &mut Vec<u64>,
) {
    if deposit.initial_quantity == 0 {
        return;
    }
    *cell_count += 1;
    *total_quantity = total_quantity.saturating_add(deposit.initial_quantity);
    quantities.push(deposit.initial_quantity);
}

fn top_decile_share_permille(quantities: &mut [u64]) -> u16 {
    if quantities.is_empty() {
        return 0;
    }
    quantities.sort_unstable_by(|left, right| right.cmp(left));
    let top_count = quantities.len().div_ceil(10).max(1);
    let total = quantities.iter().fold(0_u128, |sum, &value| sum + u128::from(value));
    let top = quantities
        .iter()
        .take(top_count)
        .fold(0_u128, |sum, &value| sum + u128::from(value));
    if total == 0 {
        return 0;
    }
    u16::try_from(top.saturating_mul(1_000) / total).unwrap_or(1_000)
}

fn ratio_permille(numerator: usize, denominator: usize) -> u16 {
    if denominator == 0 {
        return 0;
    }
    let numerator = u128::try_from(numerator).unwrap_or(u128::MAX);
    let denominator = u128::try_from(denominator).unwrap_or(1);
    u16::try_from(numerator.saturating_mul(1_000) / denominator).unwrap_or(1_000)
}

fn random_permille(seed: u64, index: usize, salt: u64) -> u16 {
    let index = u64::try_from(index).unwrap_or(u64::MAX);
    let mixed = mix64(seed ^ index.wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ salt);
    u16::try_from(mixed % 1_001).unwrap_or(0)
}

fn mix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

#[cfg(test)]
mod tests {
    use super::{generate_trial_resources, resource_balance_report, validate_resource_balance};
    use crate::generate_trial_terrain;

    #[test]
    fn same_seed_and_terrain_generate_identical_resources() {
        let terrain = generate_trial_terrain(2026).expect("terrain generates");
        let first = generate_trial_resources(2026, &terrain).expect("resources generate");
        let second = generate_trial_resources(2026, &terrain).expect("resources generate");
        assert_eq!(first, second);
    }

    #[test]
    fn different_seed_changes_resource_distribution() {
        let terrain = generate_trial_terrain(2026).expect("terrain generates");
        let first = generate_trial_resources(2026, &terrain).expect("resources generate");
        let second = generate_trial_resources(2027, &terrain).expect("resources generate");
        assert_ne!(first.cells, second.cells);
    }

    #[test]
    fn representative_seeds_pass_balance_envelope() {
        for seed in [1_u64, 2, 2026, 0xfeed_beef_dead_beef] {
            let terrain = generate_trial_terrain(seed).expect("terrain generates");
            let field = generate_trial_resources(seed, &terrain).expect("resources generate");
            let report = validate_resource_balance(&field, &terrain).expect("balance passes");
            assert!(report.total_energy_quantity > 0);
            assert!(report.total_metal_quantity > 0);
            assert!(report.total_construction_quantity > 0);
        }
    }

    #[test]
    fn ocean_cells_remain_empty() {
        let terrain = generate_trial_terrain(2026).expect("terrain generates");
        let field = generate_trial_resources(2026, &terrain).expect("resources generate");
        for (sample, cell) in terrain.samples.iter().zip(&field.cells) {
            if sample.elevation_m < terrain.sea_level_m {
                assert_eq!(*cell, simulation_model::ResourceCellState::OCEAN);
            }
        }
    }

    #[test]
    fn report_matches_land_cell_count() {
        let terrain = generate_trial_terrain(2026).expect("terrain generates");
        let field = generate_trial_resources(2026, &terrain).expect("resources generate");
        let report = resource_balance_report(&field, &terrain);
        assert_eq!(report.land_cells, terrain.land_sample_count());
    }
}
