use simulation_model::WorldState;
use simulation_save::{create_bundle, decode_bundle, EngineSnapshot, SaveKind};
use simulation_worldgen::{generate_trial_resources, generate_trial_terrain};

fn snapshot_with_resources(seed: u64) -> EngineSnapshot {
    let terrain = generate_trial_terrain(seed).expect("terrain should generate");
    let resources = generate_trial_resources(seed, &terrain).expect("resources should generate");
    let mut world = WorldState::new(seed);
    world.terrain = Some(terrain);
    world.resources = Some(resources);
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
fn extracted_stock_round_trips_exactly_and_changes_canonical_binary() {
    let mut snapshot = snapshot_with_resources(2026);
    let before = create_bundle(&snapshot, SaveKind::Manual).expect("pre-extraction state saves");

    let resources = snapshot
        .world
        .resources
        .as_mut()
        .expect("resources are authoritative");
    let deposit = resources
        .cells
        .iter_mut()
        .map(|cell| &mut cell.energy)
        .find(|deposit| deposit.remaining_quantity > 0)
        .expect("generated world contains energy deposit");
    let initial_remaining = deposit.remaining_quantity;
    let requested = (initial_remaining / 3).max(1);
    let extraction = deposit.extract(requested);
    assert!(extraction.extracted_quantity > 0);
    assert!(deposit.remaining_quantity < initial_remaining);

    let after = create_bundle(&snapshot, SaveKind::Manual).expect("post-extraction state saves");
    assert_ne!(before.state_binary, after.state_binary);
    assert_ne!(
        before
            .metadata()
            .expect("before metadata")
            .state_checksum_fnv1a64,
        after
            .metadata()
            .expect("after metadata")
            .state_checksum_fnv1a64
    );

    let (_, restored) = decode_bundle(&after).expect("post-extraction state restores");
    assert_eq!(restored.world, snapshot.world);
    assert_eq!(restored, snapshot);
}

#[test]
fn resources_without_terrain_are_rejected() {
    let terrain = generate_trial_terrain(7).expect("terrain should generate");
    let resources = generate_trial_resources(7, &terrain).expect("resources should generate");
    let mut snapshot = snapshot_with_resources(7);
    snapshot.world.terrain = None;
    snapshot.world.resources = Some(resources);
    assert!(create_bundle(&snapshot, SaveKind::Manual).is_err());
}
