#![forbid(unsafe_code)]

use serde::Serialize;
use simulation_model::{
    BiomeClass, EntityRegistry, MapPoint, RegionSurface, ReliefClass, TerrainState, WorldBounds,
    WorldState,
};

pub const RENDER_SNAPSHOT_VERSION: u32 = 6;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderSnapshot {
    pub version: u32,
    pub date: RenderDate,
    pub elapsed_days: u64,
    pub command_count: u64,
    pub event_count: u64,
    pub authoritative_digest_hex: String,
    pub world: RenderWorldSnapshot,
}

impl RenderSnapshot {
    #[must_use]
    pub fn from_world(
        world: &WorldState,
        command_count: usize,
        event_count: usize,
        authoritative_digest: u64,
    ) -> Self {
        Self::build(
            world,
            command_count,
            event_count,
            authoritative_digest,
            true,
        )
    }

    /// Builds a dynamic frame without retransmitting the immutable terrain payload.
    /// The Viewer merges this frame with the terrain received during initialization.
    #[must_use]
    pub fn from_world_dynamic(
        world: &WorldState,
        command_count: usize,
        event_count: usize,
        authoritative_digest: u64,
    ) -> Self {
        Self::build(
            world,
            command_count,
            event_count,
            authoritative_digest,
            false,
        )
    }

    fn build(
        world: &WorldState,
        command_count: usize,
        event_count: usize,
        authoritative_digest: u64,
        include_terrain: bool,
    ) -> Self {
        Self {
            version: RENDER_SNAPSHOT_VERSION,
            date: RenderDate {
                year: world.date.year,
                month: world.date.month,
                day: world.date.day,
            },
            elapsed_days: world.elapsed_days,
            command_count: u64::try_from(command_count).unwrap_or(u64::MAX),
            event_count: u64::try_from(event_count).unwrap_or(u64::MAX),
            authoritative_digest_hex: format!("{authoritative_digest:016x}"),
            world: RenderWorldSnapshot::from_world(world, include_terrain),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderDate {
    pub year: u32,
    pub month: u8,
    pub day: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderWorldSnapshot {
    pub initialized: bool,
    pub bounds: Option<RenderWorldBounds>,
    pub terrain: Option<RenderTerrainSnapshot>,
    pub regions: Vec<RenderRegionSnapshot>,
    pub human_groups: Vec<RenderHumanGroupSnapshot>,
    pub settlements: Vec<RenderIdentitySnapshot>,
    pub communities: Vec<RenderIdentitySnapshot>,
    pub political_entities: Vec<RenderIdentitySnapshot>,
    pub countries: Vec<RenderIdentitySnapshot>,
    pub cities: Vec<RenderCitySnapshot>,
}

impl RenderWorldSnapshot {
    fn from_world(world: &WorldState, include_terrain: bool) -> Self {
        let terrain = include_terrain
            .then(|| world.terrain.as_ref().map(RenderTerrainSnapshot::from))
            .flatten();
        let bounds = world
            .spatial
            .bounds
            .or_else(|| world.terrain.as_ref().map(|terrain| terrain.bounds))
            .map(RenderWorldBounds::from);
        Self {
            initialized: world.terrain.is_some() || world.spatial.is_initialized(),
            bounds,
            terrain,
            regions: world
                .spatial
                .regions
                .iter()
                .map(RenderRegionSnapshot::from)
                .collect(),
            human_groups: world
                .entities
                .human_groups
                .iter()
                .filter_map(|group| {
                    group
                        .initial
                        .as_ref()
                        .map(|initial| RenderHumanGroupSnapshot {
                            id: group.id.0.to_string(),
                            region_id: initial.region_id.0.to_string(),
                            x_m: initial.x_m,
                            z_m: initial.z_m,
                            population: initial.population.to_string(),
                            food_stock_person_days: initial.food_stock_person_days.to_string(),
                            basic_resource_stock_units: initial
                                .basic_resource_stock_units
                                .to_string(),
                            mobility_permille: initial.behavior.mobility_permille,
                            exploration_permille: initial.behavior.exploration_permille,
                            settlement_bias_permille: initial.behavior.settlement_bias_permille,
                        })
                })
                .collect(),
            settlements: world
                .entities
                .settlements
                .iter()
                .map(|record| RenderIdentitySnapshot {
                    id: record.id.0.to_string(),
                })
                .collect(),
            communities: world
                .entities
                .communities
                .iter()
                .map(|record| RenderIdentitySnapshot {
                    id: record.id.0.to_string(),
                })
                .collect(),
            political_entities: world
                .entities
                .political_entities
                .iter()
                .map(|record| RenderIdentitySnapshot {
                    id: record.id.0.to_string(),
                })
                .collect(),
            countries: world
                .entities
                .countries
                .iter()
                .map(|record| RenderIdentitySnapshot {
                    id: record.id.0.to_string(),
                })
                .collect(),
            cities: world
                .entities
                .cities
                .iter()
                .map(|record| RenderCitySnapshot {
                    id: record.id.0.to_string(),
                    country_id: record.country.map(|id| id.0.to_string()),
                    region_id: record.region.map(|id| id.0.to_string()),
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderIdentitySnapshot {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderCitySnapshot {
    pub id: String,
    pub country_id: Option<String>,
    pub region_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderHumanGroupSnapshot {
    pub id: String,
    pub region_id: String,
    pub x_m: i32,
    pub z_m: i32,
    pub population: String,
    pub food_stock_person_days: String,
    pub basic_resource_stock_units: String,
    pub mobility_permille: u16,
    pub exploration_permille: u16,
    pub settlement_bias_permille: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderTerrainSnapshot {
    pub width: u16,
    pub height: u16,
    pub spacing_m: i32,
    pub sea_level_m: i16,
    pub elevation_m: Vec<i16>,
    pub moisture_permille: Vec<u16>,
    pub relief_codes: Vec<u8>,
    pub biome_codes: Vec<u8>,
    pub downstream_indices: Vec<u32>,
    pub drainage_basin_ids: Vec<u16>,
    pub flow_accumulation: Vec<u32>,
    pub river_orders: Vec<u8>,
    pub landmass_ids: Vec<u16>,
    pub island_landmass_ids: Vec<u16>,
}

impl From<&TerrainState> for RenderTerrainSnapshot {
    fn from(terrain: &TerrainState) -> Self {
        let hydrology = terrain.derive_hydrology();
        let landmasses = terrain.derive_landmasses();
        Self {
            width: terrain.width,
            height: terrain.height,
            spacing_m: terrain.spacing_m,
            sea_level_m: terrain.sea_level_m,
            elevation_m: terrain
                .samples
                .iter()
                .map(|sample| sample.elevation_m)
                .collect(),
            moisture_permille: terrain
                .samples
                .iter()
                .map(|sample| sample.moisture_permille)
                .collect(),
            relief_codes: terrain
                .samples
                .iter()
                .map(|sample| relief_code(sample.relief))
                .collect(),
            biome_codes: terrain
                .samples
                .iter()
                .map(|sample| biome_code(sample.biome))
                .collect(),
            downstream_indices: hydrology.downstream_indices,
            drainage_basin_ids: hydrology.drainage_basin_ids,
            flow_accumulation: hydrology.flow_accumulation,
            river_orders: hydrology.river_orders,
            landmass_ids: landmasses.landmass_ids,
            island_landmass_ids: landmasses.island_landmass_ids,
        }
    }
}

const fn relief_code(relief: ReliefClass) -> u8 {
    match relief {
        ReliefClass::DeepOcean => 0,
        ReliefClass::ShallowOcean => 1,
        ReliefClass::Coast => 2,
        ReliefClass::Plains => 3,
        ReliefClass::Hills => 4,
        ReliefClass::Mountains => 5,
    }
}

const fn biome_code(biome: BiomeClass) -> u8 {
    match biome {
        BiomeClass::Ocean => 0,
        BiomeClass::Grassland => 1,
        BiomeClass::Forest => 2,
        BiomeClass::Desert => 3,
        BiomeClass::Wetland => 4,
        BiomeClass::Alpine => 5,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderWorldBounds {
    pub min_x_m: i32,
    pub max_x_m: i32,
    pub min_z_m: i32,
    pub max_z_m: i32,
}

impl From<WorldBounds> for RenderWorldBounds {
    fn from(bounds: WorldBounds) -> Self {
        Self {
            min_x_m: bounds.min_x_m,
            max_x_m: bounds.max_x_m,
            min_z_m: bounds.min_z_m,
            max_z_m: bounds.max_z_m,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderRegionSnapshot {
    pub id: String,
    pub surface: RenderRegionSurface,
    pub center: RenderMapPoint,
    pub boundary: Vec<RenderMapPoint>,
    pub neighbors: Vec<String>,
    pub legal_owner: Option<String>,
    pub controller: Option<String>,
}

impl From<&simulation_model::RegionState> for RenderRegionSnapshot {
    fn from(region: &simulation_model::RegionState) -> Self {
        Self {
            id: region.id.0.to_string(),
            surface: region.surface.into(),
            center: region.center.into(),
            boundary: region.boundary.iter().copied().map(Into::into).collect(),
            neighbors: region
                .neighbors
                .iter()
                .map(|neighbor| neighbor.0.to_string())
                .collect(),
            legal_owner: region
                .political
                .legal_owner
                .map(|country| country.0.to_string()),
            controller: region
                .political
                .controller
                .map(|country| country.0.to_string()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderRegionSurface {
    Land,
    Ocean,
}

impl From<RegionSurface> for RenderRegionSurface {
    fn from(surface: RegionSurface) -> Self {
        match surface {
            RegionSurface::Land => Self::Land,
            RegionSurface::Ocean => Self::Ocean,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderMapPoint {
    pub x_m: i32,
    pub z_m: i32,
}

impl From<MapPoint> for RenderMapPoint {
    fn from(point: MapPoint) -> Self {
        Self {
            x_m: point.x_m,
            z_m: point.z_m,
        }
    }
}

#[cfg(test)]
mod tests {
    use simulation_model::{
        BiomeClass, MapPoint, RegionId, RegionPoliticalState, RegionState, RegionSurface,
        ReliefClass, TerrainSample, TerrainState, WorldBounds, WorldSpatialState, WorldState,
        NO_DOWNSTREAM_INDEX, TERRAIN_SAMPLE_COUNT, TRIAL_REGION_COUNT,
    };

    use super::{RenderRegionSurface, RenderSnapshot, RENDER_SNAPSHOT_VERSION};

    fn sample_spatial_world() -> WorldSpatialState {
        let regions = (1..=TRIAL_REGION_COUNT)
            .map(|index| {
                let id = RegionId(u64::try_from(index).expect("id fits u64"));
                let x = i32::try_from(index).expect("index fits i32") * 100;
                let mut neighbors = Vec::new();
                if index > 1 {
                    neighbors.push(RegionId(u64::try_from(index - 1).expect("id fits u64")));
                }
                if index < TRIAL_REGION_COUNT {
                    neighbors.push(RegionId(u64::try_from(index + 1).expect("id fits u64")));
                }
                RegionState {
                    id,
                    surface: RegionSurface::Land,
                    center: MapPoint::new(x, 100),
                    boundary: vec![
                        MapPoint::new(x - 20, 80),
                        MapPoint::new(x + 20, 80),
                        MapPoint::new(x, 120),
                    ],
                    neighbors,
                    political: RegionPoliticalState::unclaimed(),
                }
            })
            .collect();
        WorldSpatialState::new_trial(
            WorldBounds::new(0, 10_000, 0, 10_000).expect("bounds valid"),
            regions,
        )
        .expect("spatial world valid")
    }

    fn sample_terrain() -> TerrainState {
        TerrainState::new_trial(vec![
            TerrainSample {
                elevation_m: -500,
                moisture_permille: 500,
                relief: ReliefClass::ShallowOcean,
                biome: BiomeClass::Ocean,
            };
            TERRAIN_SAMPLE_COUNT
        ])
        .expect("terrain valid")
    }

    #[test]
    fn uninitialized_stage_one_world_serializes_without_fake_regions() {
        let world = WorldState::new(7);
        let snapshot = RenderSnapshot::from_world(&world, 0, 0, 0x1234);
        assert_eq!(snapshot.version, RENDER_SNAPSHOT_VERSION);
        assert!(!snapshot.world.initialized);
        assert!(snapshot.world.bounds.is_none());
        assert!(snapshot.world.terrain.is_none());
        assert!(snapshot.world.regions.is_empty());
        assert_eq!(snapshot.authoritative_digest_hex, "0000000000001234");
    }

    #[test]
    fn terrain_initialization_exposes_heightfield_and_derived_geography() {
        let mut world = WorldState::new(7);
        world.terrain = Some(sample_terrain());
        let snapshot = RenderSnapshot::from_world(&world, 0, 0, 0x55);
        assert!(snapshot.world.initialized);
        assert!(snapshot.world.bounds.is_some());
        let terrain = snapshot.world.terrain.expect("terrain snapshot present");
        assert_eq!(terrain.elevation_m.len(), TERRAIN_SAMPLE_COUNT);
        assert_eq!(terrain.downstream_indices.len(), TERRAIN_SAMPLE_COUNT);
        assert_eq!(terrain.river_orders.len(), TERRAIN_SAMPLE_COUNT);
        assert_eq!(terrain.landmass_ids.len(), TERRAIN_SAMPLE_COUNT);
        assert!(terrain
            .downstream_indices
            .iter()
            .all(|&index| index == NO_DOWNSTREAM_INDEX));
        assert!(terrain.island_landmass_ids.is_empty());
        assert!(snapshot.world.regions.is_empty());
    }

    #[test]
    fn dynamic_frame_omits_static_terrain_but_keeps_world_bounds() {
        let mut world = WorldState::new(7);
        world.terrain = Some(sample_terrain());
        let snapshot = RenderSnapshot::from_world_dynamic(&world, 0, 0, 0x55);
        assert!(snapshot.world.initialized);
        assert!(snapshot.world.bounds.is_some());
        assert!(snapshot.world.terrain.is_none());
    }

    #[test]
    fn initialized_world_exposes_all_sixty_region_records() {
        let mut world = WorldState::new(7);
        world.spatial = sample_spatial_world();
        let snapshot = RenderSnapshot::from_world(&world, 2, 3, 0x55);
        assert!(snapshot.world.initialized);
        assert_eq!(snapshot.world.regions.len(), 60);
        assert_eq!(snapshot.world.regions[0].surface, RenderRegionSurface::Land);
        assert_eq!(snapshot.world.regions[59].id, "60");
    }
}
