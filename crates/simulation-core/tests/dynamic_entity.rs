use simulation_core::SimulationEngine;
use simulation_model::{
    CommandError, CommandPayload, CommandTiming, CommunityId, CountryId, EntityRef, EntityRegistry,
    EntityRegistryError, EventCategory, EventPayload, HumanGroupId, MapPoint, PoliticalEntityId,
    RegionDraft, RegionId, RegionPoliticalState, RegionState, RegionSurface, SettlementId,
    WorldBounds, WorldSpatialState,
};
use simulation_save::{create_bundle, SaveKind};

#[test]
fn civilization_origin_entities_exist_before_any_country() {
    let mut engine = SimulationEngine::new(3);
    assert!(engine.state().entities.countries.is_empty());

    for payload in [
        CommandPayload::CreateHumanGroupEntity,
        CommandPayload::CreateSettlementEntity,
        CommandPayload::CreateCommunityEntity,
        CommandPayload::CreatePoliticalEntity,
    ] {
        engine
            .submit_system_command(CommandTiming::Immediate, payload)
            .expect("pre-state lifecycle command");
        engine.tick();
    }

    assert!(
        engine
            .state()
            .entities
            .human_groups
            .contains(HumanGroupId(1))
    );
    assert!(
        engine
            .state()
            .entities
            .settlements
            .contains(SettlementId(1))
    );
    assert!(
        engine
            .state()
            .entities
            .communities
            .contains(CommunityId(1))
    );
    assert!(
        engine
            .state()
            .entities
            .political_entities
            .contains(PoliticalEntityId(1))
    );
    assert!(engine.state().entities.countries.is_empty());

    let created: Vec<_> = engine
        .event_log()
        .iter()
        .filter_map(|event| match event.payload {
            EventPayload::EntityCreated { entity } => Some(entity),
            _ => None,
        })
        .collect();
    assert_eq!(
        created,
        vec![
            EntityRef::human_group(HumanGroupId(1)),
            EntityRef::settlement(SettlementId(1)),
            EntityRef::community(CommunityId(1)),
            EntityRef::political_entity(PoliticalEntityId(1)),
        ]
    );
}

#[test]
fn lifecycle_ids_are_monotonic_non_reused_and_audited() {
    let mut engine = SimulationEngine::new(7);

    engine
        .submit_system_command(
            CommandTiming::Immediate,
            CommandPayload::CreateCountryEntity,
        )
        .expect("system create command");
    engine.tick();
    assert!(engine.state().entities.countries.contains(CountryId(1)));

    engine
        .submit_system_command(
            CommandTiming::Immediate,
            CommandPayload::RemoveCountryEntity {
                country_id: CountryId(1),
            },
        )
        .expect("system remove command");
    engine.tick();
    assert!(!engine.state().entities.countries.contains(CountryId(1)));

    engine
        .submit_system_command(
            CommandTiming::Immediate,
            CommandPayload::CreateCountryEntity,
        )
        .expect("second system create command");
    engine.tick();
    assert!(engine.state().entities.countries.contains(CountryId(2)));
    assert_eq!(engine.state().entities.countries.next_id(), 3);

    let lifecycle: Vec<_> = engine
        .event_log()
        .iter()
        .filter(|event| event.category == EventCategory::EntityLifecycle)
        .map(|event| event.payload)
        .collect();
    assert_eq!(
        lifecycle,
        vec![
            EventPayload::EntityCreated {
                entity: EntityRef::country(CountryId(1)),
            },
            EventPayload::EntityRemoved {
                entity: EntityRef::country(CountryId(1)),
            },
            EventPayload::EntityCreated {
                entity: EntityRef::country(CountryId(2)),
            },
        ]
    );
}

#[test]
fn entity_lifecycle_payloads_require_system_source() {
    let mut engine = SimulationEngine::new(1);
    let before = engine.pending_commands().len();
    assert_eq!(
        engine.submit_user_command(
            CommandTiming::Immediate,
            CommandPayload::CreateHumanGroupEntity
        ),
        Err(CommandError::InvalidSourceForPayload)
    );
    assert_eq!(engine.pending_commands().len(), before);
}

#[test]
fn referenced_country_removal_is_rejected_without_mutation() {
    let mut engine = SimulationEngine::new(11);
    engine
        .submit_system_command(
            CommandTiming::Immediate,
            CommandPayload::CreateCountryEntity,
        )
        .expect("country create");
    engine.tick();

    engine
        .submit_system_command(
            CommandTiming::Immediate,
            CommandPayload::CreateCityEntity {
                country_id: Some(CountryId(1)),
                region_id: None,
            },
        )
        .expect("city create");
    engine.tick();

    engine
        .submit_system_command(
            CommandTiming::Immediate,
            CommandPayload::RemoveCountryEntity {
                country_id: CountryId(1),
            },
        )
        .expect("removal attempt is journaled");
    engine.tick();

    assert!(engine.state().entities.countries.contains(CountryId(1)));
    assert!(engine.event_log().iter().any(|event| {
        matches!(
            event.payload,
            EventPayload::EntityMutationRejected {
                error: EntityRegistryError::ReferenceInUse
            }
        )
    }));
}

#[test]
fn pre_state_save_load_preserves_all_allocators() {
    let mut engine = SimulationEngine::new(17);
    for payload in [
        CommandPayload::CreateHumanGroupEntity,
        CommandPayload::RemoveHumanGroupEntity {
            human_group_id: HumanGroupId(1),
        },
        CommandPayload::CreateHumanGroupEntity,
        CommandPayload::CreateSettlementEntity,
        CommandPayload::CreateCommunityEntity,
        CommandPayload::CreatePoliticalEntity,
    ] {
        engine
            .submit_system_command(CommandTiming::Immediate, payload)
            .expect("pre-state lifecycle command");
        engine.tick();
    }

    let bundle = engine
        .save_bundle(SaveKind::Manual)
        .expect("pre-state entity world must save");
    let mut restored = SimulationEngine::from_save_bundle(&bundle).expect("pre-state save restores");

    assert_eq!(restored.export_snapshot(), engine.export_snapshot());
    assert_eq!(restored.state().entities.human_groups.next_id(), 3);
    assert_eq!(restored.state().entities.settlements.next_id(), 2);
    assert_eq!(restored.state().entities.communities.next_id(), 2);
    assert_eq!(restored.state().entities.political_entities.next_id(), 2);
    assert!(restored.state().entities.countries.is_empty());

    restored
        .submit_system_command(
            CommandTiming::Immediate,
            CommandPayload::CreateHumanGroupEntity,
        )
        .expect("post-restore group create");
    restored.tick();
    assert!(
        restored
            .state()
            .entities
            .human_groups
            .contains(HumanGroupId(3))
    );
}

#[test]
fn save_load_preserves_allocator_and_next_creation_identity() {
    let mut engine = SimulationEngine::new(19);
    for payload in [
        CommandPayload::CreateCountryEntity,
        CommandPayload::RemoveCountryEntity {
            country_id: CountryId(1),
        },
        CommandPayload::CreateCountryEntity,
    ] {
        engine
            .submit_system_command(CommandTiming::Immediate, payload)
            .expect("lifecycle command");
        engine.tick();
    }

    let bundle = engine
        .save_bundle(SaveKind::Manual)
        .expect("dynamic entity state must save");
    let mut restored = SimulationEngine::from_save_bundle(&bundle).expect("save must restore");
    assert_eq!(restored.export_snapshot(), engine.export_snapshot());
    assert_eq!(restored.state().entities.countries.next_id(), 3);

    restored
        .submit_system_command(
            CommandTiming::Immediate,
            CommandPayload::CreateCountryEntity,
        )
        .expect("post-restore create");
    restored.tick();
    assert!(restored.state().entities.countries.contains(CountryId(3)));
}

#[test]
fn political_entity_can_transition_to_country_without_predefining_emergence_cause() {
    let mut engine = SimulationEngine::new(21);
    engine
        .submit_system_command(
            CommandTiming::Immediate,
            CommandPayload::CreatePoliticalEntity,
        )
        .expect("political entity create");
    engine.tick();
    assert!(
        engine
            .state()
            .entities
            .political_entities
            .contains(PoliticalEntityId(1))
    );
    assert!(engine.state().entities.countries.is_empty());

    engine
        .submit_system_command(
            CommandTiming::Immediate,
            CommandPayload::PromotePoliticalEntityToCountry {
                political_entity_id: PoliticalEntityId(1),
            },
        )
        .expect("promotion command");
    engine.tick();

    assert!(engine.state().entities.political_entities.is_empty());
    assert!(engine.state().entities.countries.contains(CountryId(1)));
    assert!(engine.event_log().iter().any(|event| {
        matches!(
            event.payload,
            EventPayload::EntityTransitioned { from, to }
                if from == EntityRef::political_entity(PoliticalEntityId(1))
                    && to == EntityRef::country(CountryId(1))
        )
    }));
}

#[test]
fn pre_state_and_country_transition_journal_replay_matches_authoritative_digest() {
    let mut engine = SimulationEngine::new(23);
    for payload in [
        CommandPayload::CreateHumanGroupEntity,
        CommandPayload::CreateSettlementEntity,
        CommandPayload::CreateCommunityEntity,
        CommandPayload::CreatePoliticalEntity,
        CommandPayload::PromotePoliticalEntityToCountry {
            political_entity_id: PoliticalEntityId(1),
        },
    ] {
        engine
            .submit_system_command(CommandTiming::Immediate, payload)
            .expect("civilization-origin lifecycle command");
        engine.tick();
    }

    let journal = engine.accepted_command_journal();
    let replayed = SimulationEngine::replay_from_journal(
        engine.state().seed,
        engine.state().elapsed_days,
        &journal,
    )
    .expect("civilization-origin journal must replay");

    assert_eq!(replayed.export_snapshot(), engine.export_snapshot());
    assert_eq!(
        replayed
            .authoritative_state_digest()
            .expect("replay digest"),
        engine.authoritative_state_digest().expect("source digest")
    );
}

#[test]
fn lifecycle_command_journal_replay_matches_authoritative_digest() {
    let mut engine = SimulationEngine::new(29);
    engine
        .submit_system_command(
            CommandTiming::Immediate,
            CommandPayload::CreateCountryEntity,
        )
        .expect("create 1");
    engine.tick();
    engine
        .submit_system_command(
            CommandTiming::Immediate,
            CommandPayload::RemoveCountryEntity {
                country_id: CountryId(1),
            },
        )
        .expect("remove 1");
    engine.tick();
    engine
        .submit_system_command(
            CommandTiming::Immediate,
            CommandPayload::CreateCountryEntity,
        )
        .expect("create 2");
    engine.tick();

    let journal = engine.accepted_command_journal();
    let replayed = SimulationEngine::replay_from_journal(
        engine.state().seed,
        engine.state().elapsed_days,
        &journal,
    )
    .expect("entity lifecycle journal must replay");

    assert_eq!(replayed.export_snapshot(), engine.export_snapshot());
    assert_eq!(
        replayed
            .authoritative_state_digest()
            .expect("replay digest"),
        engine.authoritative_state_digest().expect("source digest")
    );
}

#[test]
fn region_registry_save_load_preserves_sparse_identity_and_allocator() {
    let mut snapshot = SimulationEngine::new(31).export_snapshot();
    snapshot.world.spatial = WorldSpatialState::new(
        WorldBounds::new(0, 1_000, 0, 1_000).expect("bounds"),
        vec![
            region(RegionId(1), 100, vec![RegionId(2)]),
            region(RegionId(2), 200, vec![RegionId(1)]),
        ],
    )
    .expect("initial topology");
    let bundle = create_bundle(&snapshot, SaveKind::Manual).expect("initial world saves");
    let mut engine = SimulationEngine::from_save_bundle(&bundle).expect("initial world restores");

    engine
        .submit_system_command(
            CommandTiming::Immediate,
            CommandPayload::CreateRegionEntity {
                draft: RegionDraft {
                    surface: RegionSurface::Land,
                    center: MapPoint::new(300, 100),
                    boundary: triangle(300),
                    neighbors: vec![RegionId(2)],
                    political: RegionPoliticalState::unclaimed(),
                },
            },
        )
        .expect("Region create");
    engine.tick();
    assert!(engine.state().spatial.region(RegionId(3)).is_some());

    engine
        .submit_system_command(
            CommandTiming::Immediate,
            CommandPayload::RemoveRegionEntity {
                region_id: RegionId(2),
            },
        )
        .expect("Region remove");
    engine.tick();
    assert!(engine.state().spatial.region(RegionId(2)).is_none());
    assert_eq!(engine.state().spatial.regions.next_id(), 4);

    let saved = engine
        .save_bundle(SaveKind::Manual)
        .expect("registry saves");
    let restored = SimulationEngine::from_save_bundle(&saved).expect("registry restores");
    assert_eq!(restored.state().spatial.regions.next_id(), 4);
    assert!(restored.state().spatial.region(RegionId(1)).is_some());
    assert!(restored.state().spatial.region(RegionId(2)).is_none());
    assert!(restored.state().spatial.region(RegionId(3)).is_some());
}

fn region(id: RegionId, x: i32, neighbors: Vec<RegionId>) -> RegionState {
    RegionState {
        id,
        surface: RegionSurface::Land,
        center: MapPoint::new(x, 100),
        boundary: triangle(x),
        neighbors,
        political: RegionPoliticalState::unclaimed(),
    }
}

fn triangle(x: i32) -> Vec<MapPoint> {
    vec![
        MapPoint::new(x - 20, 80),
        MapPoint::new(x + 20, 80),
        MapPoint::new(x, 120),
    ]
}
