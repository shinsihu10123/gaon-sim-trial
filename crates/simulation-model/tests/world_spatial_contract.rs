use simulation_model::{
    MapPoint, RegionId, RegionPoliticalState, RegionState, RegionSurface, WorldBounds,
    WorldSpatialError, WorldSpatialState, TRIAL_REGION_COUNT,
};

fn line_regions(count: usize) -> Vec<RegionState> {
    (1..=count)
        .map(|index| {
            let id = RegionId(u16::try_from(index).expect("fixture id fits u16"));
            let x = i32::try_from(index).expect("fixture index fits i32") * 100;
            let mut neighbors = Vec::new();
            if index > 1 {
                neighbors.push(RegionId(u16::try_from(index - 1).expect("id fits u16")));
            }
            if index < count {
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
    WorldBounds::new(0, 20_000, 0, 20_000).expect("test bounds are valid")
}

#[test]
fn standard_benchmark_sixty_is_a_fixture_not_a_core_limit() {
    let benchmark = WorldSpatialState::new(bounds(), line_regions(TRIAL_REGION_COUNT))
        .expect("standard benchmark remains valid");
    let smaller = WorldSpatialState::new(bounds(), line_regions(12))
        .expect("smaller topology must also be valid");
    let larger = WorldSpatialState::new(bounds(), line_regions(75))
        .expect("larger topology must also be valid");

    assert_eq!(benchmark.regions.len(), 60);
    assert_eq!(smaller.regions.len(), 12);
    assert_eq!(larger.regions.len(), 75);
}

#[test]
fn sparse_ids_are_lookupable_without_vector_index_identity() {
    let regions = vec![
        RegionState {
            id: RegionId(7),
            surface: RegionSurface::Land,
            center: MapPoint::new(100, 100),
            boundary: vec![
                MapPoint::new(80, 80),
                MapPoint::new(120, 80),
                MapPoint::new(100, 120),
            ],
            neighbors: vec![RegionId(19)],
            political: RegionPoliticalState::unclaimed(),
        },
        RegionState {
            id: RegionId(19),
            surface: RegionSurface::Land,
            center: MapPoint::new(200, 100),
            boundary: vec![
                MapPoint::new(180, 80),
                MapPoint::new(220, 80),
                MapPoint::new(200, 120),
            ],
            neighbors: vec![RegionId(7)],
            political: RegionPoliticalState::unclaimed(),
        },
    ];

    let spatial = WorldSpatialState::new(bounds(), regions).expect("sparse region IDs are valid");
    assert!(spatial.region(RegionId(8)).is_none());
    assert_eq!(
        spatial.region(RegionId(19)).map(|region| region.id),
        Some(RegionId(19))
    );
}

#[test]
fn neighbors_must_reference_an_existing_region() {
    let mut regions = line_regions(3);
    regions[2].neighbors.push(RegionId(60));
    assert!(matches!(
        WorldSpatialState::new(bounds(), regions),
        Err(WorldSpatialError::UnknownNeighbor {
            region: RegionId(3),
            neighbor: RegionId(60)
        })
    ));
}

#[test]
fn neighbors_must_be_strictly_sorted() {
    let mut regions = line_regions(3);
    regions[1].neighbors = vec![RegionId(3), RegionId(1)];
    assert!(matches!(
        WorldSpatialState::new(bounds(), regions),
        Err(WorldSpatialError::NonCanonicalNeighborOrder {
            region: RegionId(2)
        })
    ));
}

#[test]
fn polygon_points_must_stay_inside_world_bounds() {
    let mut regions = line_regions(3);
    regions[0].boundary[0] = MapPoint::new(-1, 80);
    assert!(matches!(
        WorldSpatialState::new(bounds(), regions),
        Err(WorldSpatialError::PointOutsideBounds {
            region: RegionId(1)
        })
    ));
}
