use simulation_model::{
    derive_terrain_effects, EntityRegistry, InitialKnowledgeProfile, MapPoint, RegionId,
    RegionPoliticalState, RegionState, RegionSurface, WorldSpatialState, WorldState,
};
use simulation_save::{
    create_bundle, decode_bundle, EngineSnapshot, SaveKind, SAVE_FORMAT_VERSION,
};
use simulation_worldgen::{
    generate_initial_human_groups, generate_trial_resources, generate_trial_terrain,
    InitialHumanGroupConfig, PermilleRange,
};

#[test]
fn initialized_human_groups_round_trip_exactly_in_save_v6() {
    let snapshot = snapshot_with_human_groups(2026, 4, 16_000);
    let bundle = create_bundle(&snapshot, SaveKind::Manual).expect("HumanGroup world saves");
    assert_eq!(SAVE_FORMAT_VERSION, 6);

    let (_, restored) = decode_bundle(&bundle).expect("HumanGroup world restores");
    assert_eq!(restored, snapshot);
    assert!(restored.world.entities.countries.is_empty());
    assert_eq!(restored.world.entities.human_groups.len(), 4);
    assert!(restored
        .world
        .entities
        .human_groups
        .iter()
        .all(|group| group.initial.is_some()));
}

#[test]
fn initial_human_group_state_changes_canonical_binary() {
    let first = snapshot_with_human_groups(2026, 4, 16_000);
    let second = snapshot_with_human_groups(2026, 4, 16_001);

    let first_bundle = create_bundle(&first, SaveKind::Manual).expect("first saves");
    let second_bundle = create_bundle(&second, SaveKind::Manual).expect("second saves");

    assert_ne!(first_bundle.state_binary, second_bundle.state_binary);
    assert_ne!(
        first_bundle
            .metadata()
            .expect("first metadata")
            .state_checksum_fnv1a64,
        second_bundle
            .metadata()
            .expect("second metadata")
            .state_checksum_fnv1a64
    );
}

fn snapshot_with_human_groups(seed: u64, group_count: usize, population: u64) -> EngineSnapshot {
    let terrain = generate_trial_terrain(seed).expect("terrain");
    let resources = generate_trial_resources(seed, &terrain).expect("resources");
    let effects = derive_terrain_effects(&terrain);
    let spatial = spatial_fixture(&terrain, &resources, &effects, 6);
    let config = InitialHumanGroupConfig {
        group_count,
        total_population: population,
        population_variation_permille: 200,
        initial_food_days_min: 25,
        initial_food_days_max: 50,
        basic_resource_units_per_person: 4,
        mobility_permille: PermilleRange { min: 100, max: 500 },
        exploration_permille: PermilleRange { min: 150, max: 650 },
        settlement_bias_permille: PermilleRange { min: 200, max: 700 },
        contact_radius_m: 300_000,
        knowledge_profile: InitialKnowledgeProfile::default(),
    };
    let groups =
        generate_initial_human_groups(seed, &config, &spatial, &terrain, &resources, &effects)
            .expect("HumanGroups");

    let mut world = WorldState::new(seed);
    world.terrain = Some(terrain);
    world.resources = Some(resources);
    world.spatial = spatial;
    world.entities.human_groups = groups;

    EngineSnapshot {
        world,
        pending_commands: Vec::new(),
        executed_commands: Vec::new(),
        events: Vec::new(),
        next_command_id: 1,
        next_event_id: 1,
    }
}

fn spatial_fixture(
    terrain: &simulation_model::TerrainState,
    resources: &simulation_model::ResourceFieldState,
    effects: &simulation_model::TerrainEffectField,
    count: usize,
) -> WorldSpatialState {
    let mut regions = Vec::new();
    for (index, (sample, resource)) in terrain.samples.iter().zip(&resources.cells).enumerate() {
        let effect = effects.sample(index).expect("effect alignment");
        if sample.elevation_m < terrain.sea_level_m
            || resource.food_capacity_tonnes_per_year == 0
            || resource.construction.accessibility_permille == 0
            || effect.carrying_capacity_people_per_km2 == 0
        {
            continue;
        }
        let x_index = index % usize::from(terrain.width);
        let z_index = index / usize::from(terrain.width);
        let x_m =
            terrain.bounds.min_x_m + i32::try_from(x_index).expect("x index") * terrain.spacing_m;
        let z_m =
            terrain.bounds.min_z_m + i32::try_from(z_index).expect("z index") * terrain.spacing_m;
        let id = RegionId(u64::try_from(regions.len() + 1).expect("Region ID"));
        regions.push(RegionState {
            id,
            surface: RegionSurface::Land,
            center: MapPoint::new(x_m, z_m),
            boundary: vec![
                MapPoint::new(x_m - 100, z_m - 100),
                MapPoint::new(x_m + 100, z_m - 100),
                MapPoint::new(x_m, z_m + 100),
            ],
            neighbors: Vec::new(),
            political: RegionPoliticalState::unclaimed(),
        });
        if regions.len() == count {
            break;
        }
    }
    assert_eq!(regions.len(), count);
    WorldSpatialState::new(terrain.bounds, regions).expect("spatial fixture")
}
