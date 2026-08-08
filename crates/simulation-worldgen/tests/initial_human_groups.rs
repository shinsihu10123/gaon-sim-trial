use simulation_model::{
    derive_terrain_effects, EntityRegistry, HumanGroupRuntimeSeed, InitialKnowledgeProfile, MapPoint,
    RegionId, RegionPoliticalState, RegionState, RegionSurface, WorldSpatialState, WorldState,
};
use simulation_worldgen::{
    generate_initial_human_groups, generate_trial_resources, generate_trial_terrain,
    initial_group_contacts, InitialHumanGroupConfig, PermilleRange,
};

#[test]
fn group_count_is_parameterized_and_total_population_is_exact() {
    for group_count in [3_usize, 5] {
        let (terrain, resources, effects, spatial) = fixture_world(2026, 8);
        let config = config(group_count, 12_345);
        let groups =
            generate_initial_human_groups(2026, &config, &spatial, &terrain, &resources, &effects)
                .expect("groups generate");

        assert_eq!(groups.len(), group_count);
        assert_eq!(
            groups
                .iter()
                .map(|group| group.initial.as_ref().expect("initialized").population)
                .sum::<u64>(),
            12_345
        );
    }
}

#[test]
fn groups_use_distinct_habitable_regions_and_valid_initial_stocks() {
    let (terrain, resources, effects, spatial) = fixture_world(2026, 8);
    let config = config(5, 20_000);
    let groups =
        generate_initial_human_groups(2026, &config, &spatial, &terrain, &resources, &effects)
            .expect("groups generate");

    let mut regions = groups
        .iter()
        .map(|group| group.initial.as_ref().expect("initialized").region_id)
        .collect::<Vec<_>>();
    regions.sort_unstable();
    regions.dedup();
    assert_eq!(regions.len(), groups.len());

    for group in groups.iter() {
        let initial = group.initial.as_ref().expect("initialized");
        let state = group.state.as_ref().expect("runtime state");
        assert!(spatial.region(initial.region_id).is_some());
        assert!(initial.food_stock_person_days >= initial.population * 20);
        assert!(initial.food_stock_person_days <= initial.population * 60);
        assert!(initial.basic_resource_stock_units > 0);
        assert!((100..=500).contains(&initial.behavior.mobility_permille));
        assert!((150..=650).contains(&initial.behavior.exploration_permille));
        assert!((200..=700).contains(&initial.behavior.settlement_bias_permille));
        assert_eq!(state.id, group.id);
        assert_eq!(state.region_id, initial.region_id);
        assert_eq!(state.population, initial.population);
        assert_eq!(state.mobility_permille, initial.behavior.mobility_permille);
        assert_eq!(state.nutrition_permille, config.runtime_seed.nutrition_permille);
        assert_eq!(state.cohesion_permille, config.runtime_seed.cohesion_permille);
        assert_eq!(state.risk_permille, config.runtime_seed.risk_permille);
    }
}

#[test]
fn initialization_leaves_modern_state_systems_inactive() {
    let (terrain, resources, effects, spatial) = fixture_world(2026, 8);
    let config = config(5, 15_000);
    let groups =
        generate_initial_human_groups(2026, &config, &spatial, &terrain, &resources, &effects)
            .expect("groups generate");

    let mut world = WorldState::new(2026);
    world.spatial = spatial;
    world.terrain = Some(terrain);
    world.resources = Some(resources);
    world.entities.human_groups = groups;

    assert!(!world.entities.human_groups.is_empty());
    assert!(world.entities.countries.is_empty());
    assert!(world.entities.cities.is_empty());
    assert!(world.entities.settlements.is_empty());
    assert!(world.entities.communities.is_empty());
    assert!(world.entities.political_entities.is_empty());
    assert!(world.spatial.regions.iter().all(|region| {
        region.political.legal_owner.is_none() && region.political.controller.is_none()
    }));
}

#[test]
fn same_seed_reproduces_groups_while_other_seed_changes_initialization() {
    let (terrain, resources, effects, spatial) = fixture_world(2026, 8);
    let config = config(5, 18_000);
    let first =
        generate_initial_human_groups(2026, &config, &spatial, &terrain, &resources, &effects)
            .expect("first generation");
    let second =
        generate_initial_human_groups(2026, &config, &spatial, &terrain, &resources, &effects)
            .expect("second generation");
    let changed =
        generate_initial_human_groups(2027, &config, &spatial, &terrain, &resources, &effects)
            .expect("changed seed generation");

    assert_eq!(first, second);
    assert_ne!(first, changed);
}

#[test]
fn contact_matrix_covers_every_pair_and_uses_distance_threshold() {
    let (terrain, resources, effects, spatial) = fixture_world(2026, 8);
    let config = config(5, 15_000);
    let groups =
        generate_initial_human_groups(2026, &config, &spatial, &terrain, &resources, &effects)
            .expect("groups generate");
    let contacts = initial_group_contacts(&groups, config.contact_radius_m);

    assert_eq!(contacts.len(), 5 * 4 / 2);
    assert!(contacts.iter().all(|contact| {
        contact.contact_possible == (contact.distance_m <= u64::from(config.contact_radius_m))
    }));
}

fn config(group_count: usize, total_population: u64) -> InitialHumanGroupConfig {
    InitialHumanGroupConfig {
        group_count,
        total_population,
        population_variation_permille: 250,
        initial_food_days_min: 20,
        initial_food_days_max: 60,
        basic_resource_units_per_person: 5,
        mobility_permille: PermilleRange { min: 100, max: 500 },
        exploration_permille: PermilleRange { min: 150, max: 650 },
        settlement_bias_permille: PermilleRange { min: 200, max: 700 },
        runtime_seed: HumanGroupRuntimeSeed {
            nutrition_permille: 1_000,
            cohesion_permille: 500,
            risk_permille: 500,
        },
        contact_radius_m: 400_000,
        knowledge_profile: InitialKnowledgeProfile::default(),
    }
}

fn fixture_world(
    seed: u64,
    region_count: usize,
) -> (
    simulation_model::TerrainState,
    simulation_model::ResourceFieldState,
    simulation_model::TerrainEffectField,
    WorldSpatialState,
) {
    let terrain = generate_trial_terrain(seed).expect("terrain");
    let resources = generate_trial_resources(seed, &terrain).expect("resources");
    let effects = derive_terrain_effects(&terrain);

    let mut regions = Vec::new();
    for (index, (sample, resource)) in terrain.samples.iter().zip(&resources.cells).enumerate() {
        let effect = effects.sample(index).expect("aligned effect");
        if sample.elevation_m < terrain.sea_level_m
            || resource.food_capacity_tonnes_per_year == 0
            || effect.carrying_capacity_people_per_km2 == 0
            || resource.construction.accessibility_permille == 0
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
        if regions.len() == region_count {
            break;
        }
    }
    assert_eq!(regions.len(), region_count, "fixture needs enough habitat");
    let spatial = WorldSpatialState::new(terrain.bounds, regions).expect("valid spatial fixture");
    (terrain, resources, effects, spatial)
}