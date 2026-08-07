use simulation_model::{
    MapPoint, RegionId, RegionPoliticalState, RegionState, RegionSurface, WorldBounds,
    WorldSpatialError, WorldSpatialState, TRIAL_REGION_COUNT,
};

fn trial_regions() -> Vec<RegionState> {
    (1..=TRIAL_REGION_COUNT)
        .map(|index| {
            let id = RegionId(u16::try_from(index).expect("trial id fits u16"));
            let x = i32::try_from(index).expect("trial index fits i32") * 100;
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
        .collect()
}

fn bounds() -> WorldBounds {
    WorldBounds::new(0, 10_000, 0, 10_000).expect("test bounds are valid")
}

#[test]
fn wrong_region_count_is_rejected() {
    let mut regions = trial_regions();
    regions.pop();
    assert!(matches!(
        WorldSpatialState::new_trial(bounds(), regions),
        Err(WorldSpatialError::WrongRegionCount { found: 59 })
    ));
}

#[test]
fn neighbors_must_be_strictly_sorted() {
    let mut regions = trial_regions();
    regions[1].neighbors = vec![RegionId(3), RegionId(1)];
    assert!(matches!(
        WorldSpatialState::new_trial(bounds(), regions),
        Err(WorldSpatialError::NonCanonicalNeighborOrder {
            region: RegionId(2)
        })
    ));
}

#[test]
fn polygon_points_must_stay_inside_world_bounds() {
    let mut regions = trial_regions();
    regions[0].boundary[0] = MapPoint::new(-1, 80);
    assert!(matches!(
        WorldSpatialState::new_trial(bounds(), regions),
        Err(WorldSpatialError::PointOutsideBounds {
            region: RegionId(1)
        })
    ));
}
