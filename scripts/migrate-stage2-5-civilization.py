from pathlib import Path

path = Path("crates/simulation-save/src/binary.rs")
text = path.read_text()

replacements = [
    (
        """use simulation_model::{
    BiomeClass, CityEntity, CityId, CityRegistry, CommandExecutionRecord, CommandId,
    CommandPayload, CommandPriority, CommandSource, CountryEntity, CountryId, CountryRegistry,
    EntityKind, EntityRef, EntityRegistry, EntityRegistryError, EntityWorldState, EventCategory,
    EventId, EventPayload, EventRecord, EventSource, MapPoint, QueuedCommand, RegionDraft,
    RegionId, RegionPoliticalState, RegionRegistry, RegionState, RegionSurface, ReliefClass,
    ResourceCellState, ResourceDeposit, ResourceFieldState, SimulationDate, StableEntityId,
    TerrainSample, TerrainState, WorldBounds, WorldSpatialState, WorldState,
};""",
        """use simulation_model::{
    BiomeClass, CityEntity, CityId, CityRegistry, CommandExecutionRecord, CommandId,
    CommandPayload, CommandPriority, CommandSource, CommunityEntity, CommunityId,
    CommunityRegistry, CountryEntity, CountryId, CountryRegistry, EntityKind, EntityRef,
    EntityRegistry, EntityRegistryError, EntityWorldState, EventCategory, EventId, EventPayload,
    EventRecord, EventSource, HumanGroupEntity, HumanGroupId, HumanGroupRegistry, MapPoint,
    PoliticalEntity, PoliticalEntityId, PoliticalEntityRegistry, QueuedCommand, RegionDraft,
    RegionId, RegionPoliticalState, RegionRegistry, RegionState, RegionSurface, ReliefClass,
    ResourceCellState, ResourceDeposit, ResourceFieldState, SettlementEntity, SettlementId,
    SettlementRegistry, SimulationDate, StableEntityId, TerrainSample, TerrainState, WorldBounds,
    WorldSpatialState, WorldState,
};""",
    ),
    (
        """    write_u64(bytes, world.entities.countries.next_id());
    write_count(bytes, \"countries\", world.entities.countries.len())?;""",
        """    write_u64(bytes, world.entities.human_groups.next_id());
    write_count(bytes, \"human_groups\", world.entities.human_groups.len())?;
    for group in world.entities.human_groups.iter() {
        write_u64(bytes, group.id.0);
    }

    write_u64(bytes, world.entities.settlements.next_id());
    write_count(bytes, \"settlements\", world.entities.settlements.len())?;
    for settlement in world.entities.settlements.iter() {
        write_u64(bytes, settlement.id.0);
    }

    write_u64(bytes, world.entities.communities.next_id());
    write_count(bytes, \"communities\", world.entities.communities.len())?;
    for community in world.entities.communities.iter() {
        write_u64(bytes, community.id.0);
    }

    write_u64(bytes, world.entities.political_entities.next_id());
    write_count(
        bytes,
        \"political_entities\",
        world.entities.political_entities.len(),
    )?;
    for political_entity in world.entities.political_entities.iter() {
        write_u64(bytes, political_entity.id.0);
    }

    write_u64(bytes, world.entities.countries.next_id());
    write_count(bytes, \"countries\", world.entities.countries.len())?;""",
    ),
    (
        """    let country_next_id = cursor.read_u64()?;
    let country_count = cursor.read_count(\"countries\")?;""",
        """    let human_group_next_id = cursor.read_u64()?;
    let human_group_count = cursor.read_count(\"human_groups\")?;
    let mut human_groups = Vec::with_capacity(human_group_count);
    for _ in 0..human_group_count {
        human_groups.push(HumanGroupEntity {
            id: HumanGroupId(cursor.read_u64()?),
        });
    }
    let human_groups = HumanGroupRegistry::from_parts(human_group_next_id, human_groups)
        .map_err(|_| SaveError::InvalidBinary(\"invalid HumanGroup registry\"))?;

    let settlement_next_id = cursor.read_u64()?;
    let settlement_count = cursor.read_count(\"settlements\")?;
    let mut settlements = Vec::with_capacity(settlement_count);
    for _ in 0..settlement_count {
        settlements.push(SettlementEntity {
            id: SettlementId(cursor.read_u64()?),
        });
    }
    let settlements = SettlementRegistry::from_parts(settlement_next_id, settlements)
        .map_err(|_| SaveError::InvalidBinary(\"invalid Settlement registry\"))?;

    let community_next_id = cursor.read_u64()?;
    let community_count = cursor.read_count(\"communities\")?;
    let mut communities = Vec::with_capacity(community_count);
    for _ in 0..community_count {
        communities.push(CommunityEntity {
            id: CommunityId(cursor.read_u64()?),
        });
    }
    let communities = CommunityRegistry::from_parts(community_next_id, communities)
        .map_err(|_| SaveError::InvalidBinary(\"invalid Community registry\"))?;

    let political_entity_next_id = cursor.read_u64()?;
    let political_entity_count = cursor.read_count(\"political_entities\")?;
    let mut political_entities = Vec::with_capacity(political_entity_count);
    for _ in 0..political_entity_count {
        political_entities.push(PoliticalEntity {
            id: PoliticalEntityId(cursor.read_u64()?),
        });
    }
    let political_entities =
        PoliticalEntityRegistry::from_parts(political_entity_next_id, political_entities)
            .map_err(|_| SaveError::InvalidBinary(\"invalid PoliticalEntity registry\"))?;

    let country_next_id = cursor.read_u64()?;
    let country_count = cursor.read_count(\"countries\")?;""",
    ),
    (
        """        entities: EntityWorldState { countries, cities },""",
        """        entities: EntityWorldState {
            human_groups,
            settlements,
            communities,
            political_entities,
            countries,
            cities,
        },""",
    ),
    (
        """        CommandPayload::RemoveRegionEntity { region_id } => {
            write_u8(bytes, 6);
            write_u64(bytes, region_id.0);
        }
    }""",
        """        CommandPayload::RemoveRegionEntity { region_id } => {
            write_u8(bytes, 6);
            write_u64(bytes, region_id.0);
        }
        CommandPayload::CreateHumanGroupEntity => write_u8(bytes, 7),
        CommandPayload::RemoveHumanGroupEntity { human_group_id } => {
            write_u8(bytes, 8);
            write_u64(bytes, human_group_id.0);
        }
        CommandPayload::CreateSettlementEntity => write_u8(bytes, 9),
        CommandPayload::RemoveSettlementEntity { settlement_id } => {
            write_u8(bytes, 10);
            write_u64(bytes, settlement_id.0);
        }
        CommandPayload::CreateCommunityEntity => write_u8(bytes, 11),
        CommandPayload::RemoveCommunityEntity { community_id } => {
            write_u8(bytes, 12);
            write_u64(bytes, community_id.0);
        }
        CommandPayload::CreatePoliticalEntity => write_u8(bytes, 13),
        CommandPayload::RemovePoliticalEntity {
            political_entity_id,
        } => {
            write_u8(bytes, 14);
            write_u64(bytes, political_entity_id.0);
        }
    }""",
    ),
    (
        """        6 => Ok(CommandPayload::RemoveRegionEntity {
            region_id: RegionId(cursor.read_u64()?),
        }),
        tag => Err(SaveError::InvalidTag {""",
        """        6 => Ok(CommandPayload::RemoveRegionEntity {
            region_id: RegionId(cursor.read_u64()?),
        }),
        7 => Ok(CommandPayload::CreateHumanGroupEntity),
        8 => Ok(CommandPayload::RemoveHumanGroupEntity {
            human_group_id: HumanGroupId(cursor.read_u64()?),
        }),
        9 => Ok(CommandPayload::CreateSettlementEntity),
        10 => Ok(CommandPayload::RemoveSettlementEntity {
            settlement_id: SettlementId(cursor.read_u64()?),
        }),
        11 => Ok(CommandPayload::CreateCommunityEntity),
        12 => Ok(CommandPayload::RemoveCommunityEntity {
            community_id: CommunityId(cursor.read_u64()?),
        }),
        13 => Ok(CommandPayload::CreatePoliticalEntity),
        14 => Ok(CommandPayload::RemovePoliticalEntity {
            political_entity_id: PoliticalEntityId(cursor.read_u64()?),
        }),
        tag => Err(SaveError::InvalidTag {""",
    ),
    (
        """        match entity.kind {
            EntityKind::Country => 0,
            EntityKind::Region => 1,
            EntityKind::City => 2,
        },""",
        """        match entity.kind {
            EntityKind::Country => 0,
            EntityKind::Region => 1,
            EntityKind::City => 2,
            EntityKind::HumanGroup => 3,
            EntityKind::Settlement => 4,
            EntityKind::Community => 5,
            EntityKind::PoliticalEntity => 6,
        },""",
    ),
    (
        """        0 => EntityKind::Country,
        1 => EntityKind::Region,
        2 => EntityKind::City,
        tag => {""",
        """        0 => EntityKind::Country,
        1 => EntityKind::Region,
        2 => EntityKind::City,
        3 => EntityKind::HumanGroup,
        4 => EntityKind::Settlement,
        5 => EntityKind::Community,
        6 => EntityKind::PoliticalEntity,
        tag => {""",
    ),
]

for old, new in replacements:
    if old not in text:
        raise SystemExit(f"migration anchor missing:\n{old[:160]}")
    text = text.replace(old, new, 1)

path.write_text(text)
