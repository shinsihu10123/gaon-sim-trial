use simulation_model::{
    MapPoint, RegionId, RegionPoliticalState, RegionState, RegionSurface, WorldBounds,
    WorldSpatialState, WorldState, TRIAL_REGION_COUNT,
};
use simulation_save::{checksum_fnv1a64, create_bundle, EngineSnapshot, SaveKind};

fn trial_spatial() -> WorldSpatialState {
    let regions = (1..=TRIAL_REGION_COUNT)
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
        .collect();

    WorldSpatialState::new_trial(
        WorldBounds::new(0, 10_000, 0, 10_000).expect("valid bounds"),
        regions,
    )
    .expect("valid trial topology")
}

fn snapshot(world: WorldState) -> EngineSnapshot {
    EngineSnapshot {
        world,
        pending_commands: Vec::new(),
        executed_commands: Vec::new(),
        events: Vec::new(),
        next_command_id: 1,
        next_event_id: 1,
    }
}

#[test]
fn initialized_spatial_state_changes_canonical_binary_and_checksum() {
    let uninitialized = create_bundle(&snapshot(WorldState::new(77)), SaveKind::Manual)
        .expect("uninitialized Stage 1 world remains valid");

    let mut initialized_world = WorldState::new(77);
    initialized_world.spatial = trial_spatial();
    let initialized = create_bundle(&snapshot(initialized_world), SaveKind::Manual)
        .expect("initialized Stage 2.1 world should save");

    assert_ne!(uninitialized.state_binary, initialized.state_binary);
    assert_ne!(
        checksum_fnv1a64(&uninitialized.state_binary),
        checksum_fnv1a64(&initialized.state_binary)
    );
}
