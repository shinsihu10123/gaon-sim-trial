use simulation_model::{
    derive_terrain_effects, InitialKnowledgeProfile, MapPoint, RegionId, RegionPoliticalState,
    RegionState, RegionSurface, WorldSpatialState, WorldState, TRIAL_REGION_COUNT,
};

use crate::{
    generate_initial_human_groups, generate_trial_resources, generate_trial_terrain,
    HumanGroupGenerationError, InitialHumanGroupConfig, PermilleRange, ResourceGenerationError,
};

const BENCHMARK_REGION_COLUMNS: usize = 10;
const BENCHMARK_REGION_ROWS: usize = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrialBootstrapError {
    Terrain,
    Resources,
    Spatial,
    HumanGroups(HumanGroupGenerationError),
}

impl core::fmt::Display for TrialBootstrapError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Terrain => formatter.write_str("trial terrain generation failed"),
            Self::Resources => formatter.write_str("trial resource generation failed"),
            Self::Spatial => formatter.write_str("trial Region bootstrap failed"),
            Self::HumanGroups(error) => {
                write!(formatter, "trial HumanGroup bootstrap failed: {error}")
            }
        }
    }
}

impl std::error::Error for TrialBootstrapError {}

/// Explicit Viewer/headless preview preset. These values are scenario inputs,
/// not Simulation Core invariants.
#[must_use]
pub fn trial_preview_human_group_config() -> InitialHumanGroupConfig {
    InitialHumanGroupConfig {
        group_count: 8,
        total_population: 12_000,
        population_variation_permille: 250,
        initial_food_days_min: 20,
        initial_food_days_max: 60,
        basic_resource_units_per_person: 5,
        mobility_permille: PermilleRange { min: 100, max: 500 },
        exploration_permille: PermilleRange { min: 150, max: 650 },
        settlement_bias_permille: PermilleRange { min: 200, max: 700 },
        contact_radius_m: 400_000,
        knowledge_profile: InitialKnowledgeProfile::default(),
    }
}

/// Builds the common civilization-origin world used by browser WASM and the
/// headless runner. The benchmark Region count is a preset only; registries
/// remain dynamically sized after initialization.
///
/// # Errors
/// Returns [`TrialBootstrapError`] when any deterministic generation stage fails.
pub fn generate_trial_civilization_origin_world(
    seed: u64,
    human_group_config: &InitialHumanGroupConfig,
) -> Result<WorldState, TrialBootstrapError> {
    let terrain = generate_trial_terrain(seed).map_err(|_| TrialBootstrapError::Terrain)?;
    let resources = generate_trial_resources(seed, &terrain)
        .map_err(|_: ResourceGenerationError| TrialBootstrapError::Resources)?;
    let effects = derive_terrain_effects(&terrain);
    let spatial = benchmark_regions_from_terrain(&terrain)?;
    let human_groups = generate_initial_human_groups(
        seed,
        human_group_config,
        &spatial,
        &terrain,
        &resources,
        &effects,
    )
    .map_err(TrialBootstrapError::HumanGroups)?;

    let mut world = WorldState::new(seed);
    world.terrain = Some(terrain);
    world.resources = Some(resources);
    world.spatial = spatial;
    world.entities.human_groups = human_groups;
    Ok(world)
}

fn benchmark_regions_from_terrain(
    terrain: &simulation_model::TerrainState,
) -> Result<WorldSpatialState, TrialBootstrapError> {
    debug_assert_eq!(
        BENCHMARK_REGION_COLUMNS * BENCHMARK_REGION_ROWS,
        TRIAL_REGION_COUNT
    );
    let intervals_x = usize::from(terrain.width).saturating_sub(1);
    let intervals_z = usize::from(terrain.height).saturating_sub(1);
    let mut regions = Vec::with_capacity(TRIAL_REGION_COUNT);

    for row in 0..BENCHMARK_REGION_ROWS {
        for column in 0..BENCHMARK_REGION_COLUMNS {
            let x0 = column * intervals_x / BENCHMARK_REGION_COLUMNS;
            let x1 = (column + 1) * intervals_x / BENCHMARK_REGION_COLUMNS;
            let z0 = row * intervals_z / BENCHMARK_REGION_ROWS;
            let z1 = (row + 1) * intervals_z / BENCHMARK_REGION_ROWS;
            let center_x = (x0 + x1) / 2;
            let center_z = (z0 + z1) / 2;
            let center_index = center_z * usize::from(terrain.width) + center_x;
            let center_sample = terrain
                .samples
                .get(center_index)
                .ok_or(TrialBootstrapError::Spatial)?;
            let id = RegionId(
                u64::try_from(row * BENCHMARK_REGION_COLUMNS + column + 1)
                    .map_err(|_| TrialBootstrapError::Spatial)?,
            );
            let mut neighbors = Vec::with_capacity(4);
            if column > 0 {
                neighbors.push(RegionId(id.0 - 1));
            }
            if column + 1 < BENCHMARK_REGION_COLUMNS {
                neighbors.push(RegionId(id.0 + 1));
            }
            if row > 0 {
                neighbors.push(RegionId(
                    id.0 - u64::try_from(BENCHMARK_REGION_COLUMNS).unwrap_or(10),
                ));
            }
            if row + 1 < BENCHMARK_REGION_ROWS {
                neighbors.push(RegionId(
                    id.0 + u64::try_from(BENCHMARK_REGION_COLUMNS).unwrap_or(10),
                ));
            }
            neighbors.sort_unstable();

            let point = |x_index: usize, z_index: usize| -> Result<MapPoint, TrialBootstrapError> {
                let x_offset = i32::try_from(x_index).map_err(|_| TrialBootstrapError::Spatial)?;
                let z_offset = i32::try_from(z_index).map_err(|_| TrialBootstrapError::Spatial)?;
                Ok(MapPoint::new(
                    terrain.bounds.min_x_m + x_offset * terrain.spacing_m,
                    terrain.bounds.min_z_m + z_offset * terrain.spacing_m,
                ))
            };

            regions.push(RegionState {
                id,
                surface: if center_sample.elevation_m >= terrain.sea_level_m {
                    RegionSurface::Land
                } else {
                    RegionSurface::Ocean
                },
                center: point(center_x, center_z)?,
                boundary: vec![
                    point(x0, z0)?,
                    point(x1, z0)?,
                    point(x1, z1)?,
                    point(x0, z1)?,
                ],
                neighbors,
                political: RegionPoliticalState::unclaimed(),
            });
        }
    }

    WorldSpatialState::new(terrain.bounds, regions).map_err(|_| TrialBootstrapError::Spatial)
}

#[cfg(test)]
mod tests {
    use simulation_model::EntityRegistry;

    use super::{generate_trial_civilization_origin_world, trial_preview_human_group_config};

    #[test]
    fn shared_preview_bootstrap_contains_regions_and_pre_state_human_groups() {
        let world =
            generate_trial_civilization_origin_world(2026, &trial_preview_human_group_config())
                .expect("bootstrap world");
        assert_eq!(world.spatial.regions.len(), 60);
        assert_eq!(world.entities.human_groups.len(), 8);
        assert!(world.entities.countries.is_empty());
        assert!(world.entities.cities.is_empty());
        assert!(world
            .spatial
            .regions
            .iter()
            .all(|region| region.political.legal_owner.is_none()
                && region.political.controller.is_none()));
    }

    #[test]
    fn shared_preview_bootstrap_is_deterministic() {
        let config = trial_preview_human_group_config();
        let first = generate_trial_civilization_origin_world(2026, &config).expect("first");
        let second = generate_trial_civilization_origin_world(2026, &config).expect("second");
        assert_eq!(first, second);
    }
}
