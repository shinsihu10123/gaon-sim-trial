use simulation_model::{
    BiomeClass, ReliefClass, TerrainSample, TerrainState, WorldState, TERRAIN_SAMPLE_COUNT,
};
use simulation_save::{checksum_fnv1a64, create_bundle, EngineSnapshot, SaveKind};

fn terrain(elevation_m: i16) -> TerrainState {
    TerrainState::new_trial(vec![
        TerrainSample {
            elevation_m,
            moisture_permille: 500,
            relief: ReliefClass::ShallowOcean,
            biome: BiomeClass::Ocean,
        };
        TERRAIN_SAMPLE_COUNT
    ])
    .expect("test terrain should validate")
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
fn terrain_sample_change_changes_canonical_binary_and_checksum() {
    let mut first_world = WorldState::new(77);
    first_world.terrain = Some(terrain(-500));
    let first = create_bundle(&snapshot(first_world), SaveKind::Manual)
        .expect("first terrain should save");

    let mut second_world = WorldState::new(77);
    second_world.terrain = Some(terrain(-600));
    let second = create_bundle(&snapshot(second_world), SaveKind::Manual)
        .expect("second terrain should save");

    assert_ne!(first.state_binary, second.state_binary);
    assert_ne!(
        checksum_fnv1a64(&first.state_binary),
        checksum_fnv1a64(&second.state_binary)
    );
}
