use simulation_model::{
    EntityRegistry, HumanGroupBehaviorProfile, HumanGroupInitialState, HumanGroupRuntimeSeed,
    InitialKnowledgeProfile, MapPoint, RegionId, RegionPoliticalState, RegionState, RegionSurface,
    WorldSpatialState, WorldState,
};
use simulation_protocol::{RenderSnapshot, RENDER_SNAPSHOT_VERSION};

#[test]
fn zero_country_world_exposes_authoritative_human_group_snapshot() {
    let mut world = WorldState::new(2026);
    world.spatial = WorldSpatialState::new(
        simulation_model::WorldBounds::new(0, 1_000, 0, 1_000).expect("bounds"),
        vec![RegionState {
            id: RegionId(1),
            surface: RegionSurface::Land,
            center: MapPoint::new(500, 500),
            boundary: vec![
                MapPoint::new(400, 400),
                MapPoint::new(600, 400),
                MapPoint::new(500, 600),
            ],
            neighbors: Vec::new(),
            political: RegionPoliticalState::unclaimed(),
        }],
    )
    .expect("spatial world");
    let id = world
        .entities
        .human_groups
        .create_initialized_with_runtime(
            HumanGroupInitialState {
                region_id: RegionId(1),
                x_m: 500,
                z_m: 500,
                terrain_sample_index: 0,
                population: 1_234,
                food_stock_person_days: 37_020,
                basic_resource_stock_units: 4_000,
                behavior: HumanGroupBehaviorProfile {
                    mobility_permille: 350,
                    exploration_permille: 420,
                    settlement_bias_permille: 510,
                },
                knowledge: InitialKnowledgeProfile::default(),
            },
            HumanGroupRuntimeSeed {
                nutrition_permille: 910,
                cohesion_permille: 620,
                risk_permille: 140,
            },
        )
        .expect("initialized runtime group");
    assert!(world.entities.countries.is_empty());

    let snapshot = RenderSnapshot::from_world(&world, 0, 0, 0x55);
    assert_eq!(RENDER_SNAPSHOT_VERSION, 7);
    assert_eq!(snapshot.world.human_groups.len(), 1);
    let group = &snapshot.world.human_groups[0];
    assert_eq!(group.id, id.0.to_string());
    assert_eq!(group.region_id, "1");
    assert_eq!(group.population, "1234");
    assert_eq!(group.food_stock_person_days, "37020");
    assert_eq!(group.basic_resource_stock_units, "4000");
    assert_eq!(group.x_m, 500);
    assert_eq!(group.z_m, 500);
    assert_eq!(group.mobility_permille, 350);
    assert_eq!(group.nutrition_permille, 910);
    assert_eq!(group.cohesion_permille, 620);
    assert_eq!(group.risk_permille, 140);
}
