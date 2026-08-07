#![forbid(unsafe_code)]

use serde::Serialize;
use simulation_model::{MapPoint, RegionSurface, WorldBounds, WorldState};

pub const RENDER_SNAPSHOT_VERSION: u32 = 1;

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
            world: RenderWorldSnapshot::from_world(world),
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
    pub regions: Vec<RenderRegionSnapshot>,
}

impl RenderWorldSnapshot {
    fn from_world(world: &WorldState) -> Self {
        Self {
            initialized: world.spatial.is_initialized(),
            bounds: world.spatial.bounds.map(RenderWorldBounds::from),
            regions: world
                .spatial
                .regions
                .iter()
                .map(RenderRegionSnapshot::from)
                .collect(),
        }
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
    pub id: u16,
    pub surface: RenderRegionSurface,
    pub center: RenderMapPoint,
    pub boundary: Vec<RenderMapPoint>,
    pub neighbors: Vec<u16>,
    pub legal_owner: Option<u16>,
    pub controller: Option<u16>,
}

impl From<&simulation_model::RegionState> for RenderRegionSnapshot {
    fn from(region: &simulation_model::RegionState) -> Self {
        Self {
            id: region.id.0,
            surface: region.surface.into(),
            center: region.center.into(),
            boundary: region.boundary.iter().copied().map(Into::into).collect(),
            neighbors: region.neighbors.iter().map(|neighbor| neighbor.0).collect(),
            legal_owner: region.political.legal_owner.map(|country| country.0),
            controller: region.political.controller.map(|country| country.0),
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
        MapPoint, RegionId, RegionPoliticalState, RegionState, RegionSurface, WorldBounds,
        WorldSpatialState, WorldState, TRIAL_REGION_COUNT,
    };

    use super::{RenderRegionSurface, RenderSnapshot};

    fn sample_spatial_world() -> WorldSpatialState {
        let regions = (1..=TRIAL_REGION_COUNT)
            .map(|index| {
                let id = RegionId(u16::try_from(index).expect("id fits u16"));
                let x = i32::try_from(index).expect("index fits i32") * 100;
                let mut neighbors = Vec::new();
                if index > 1 {
                    neighbors.push(RegionId(u16::try_from(index - 1).expect("id fits u16")));
                }
                if index < TRIAL_REGION_COUNT {
                    neighbors.push(RegionId(u16::try_from(index + 1).expect("id fits u16")));
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

    #[test]
    fn uninitialized_stage_one_world_serializes_without_fake_regions() {
        let world = WorldState::new(7);
        let snapshot = RenderSnapshot::from_world(&world, 0, 0, 0x1234);
        assert!(!snapshot.world.initialized);
        assert!(snapshot.world.bounds.is_none());
        assert!(snapshot.world.regions.is_empty());
        assert_eq!(snapshot.authoritative_digest_hex, "0000000000001234");
    }

    #[test]
    fn initialized_world_exposes_all_sixty_region_records() {
        let mut world = WorldState::new(7);
        world.spatial = sample_spatial_world();
        let snapshot = RenderSnapshot::from_world(&world, 2, 3, 0x55);
        assert!(snapshot.world.initialized);
        assert_eq!(snapshot.world.regions.len(), 60);
        assert_eq!(snapshot.world.regions[0].surface, RenderRegionSurface::Land);
        assert_eq!(snapshot.world.regions[59].id, 60);
    }
}
