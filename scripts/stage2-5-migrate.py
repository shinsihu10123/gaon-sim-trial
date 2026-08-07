from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    target = Path(path)
    text = target.read_text()
    if old not in text:
        raise SystemExit(f"anchor not found in {path}: {old[:120]!r}")
    target.write_text(text.replace(old, new, 1))


# World: removal must be atomic.
replace_once(
    "crates/simulation-model/src/world.rs",
    """    pub fn remove_region(&mut self, id: RegionId) -> Result<RegionState, WorldSpatialError> {
        let removed = self
            .regions
            .remove(id)
            .map_err(|_| WorldSpatialError::UnknownRegion { region: id })?;
        if self.regions.is_empty() {
            return Err(WorldSpatialError::PartialInitialization);
        }
        self.validate()?;
        Ok(removed)
    }
""",
    """    pub fn remove_region(&mut self, id: RegionId) -> Result<RegionState, WorldSpatialError> {
        if self.regions.len() <= 1 {
            return Err(WorldSpatialError::PartialInitialization);
        }
        let removed = self
            .regions
            .remove(id)
            .map_err(|_| WorldSpatialError::UnknownRegion { region: id })?;
        self.validate()?;
        Ok(removed)
    }
""",
)

# Model command journal becomes the deterministic entity lifecycle journal.
replace_once(
    "crates/simulation-model/src/lib.rs",
    """pub enum CommandSource {
    User,
    CountryAi(CountryId),
}
""",
    """pub enum CommandSource {
    System,
    User,
    CountryAi(CountryId),
}
""",
)
replace_once(
    "crates/simulation-model/src/lib.rs",
    """pub enum CommandPriority {
    UserIntervention = 10,
    CountryAi = 20,
}

impl CommandPriority {
    #[must_use]
    pub const fn for_source(source: CommandSource) -> Self {
        match source {
            CommandSource::User => Self::UserIntervention,
            CommandSource::CountryAi(_) => Self::CountryAi,
        }
    }
}
""",
    """pub enum CommandPriority {
    System = 0,
    UserIntervention = 10,
    CountryAi = 20,
}

impl CommandPriority {
    #[must_use]
    pub const fn for_source(source: CommandSource) -> Self {
        match source {
            CommandSource::System => Self::System,
            CommandSource::User => Self::UserIntervention,
            CommandSource::CountryAi(_) => Self::CountryAi,
        }
    }
}
""",
)
replace_once(
    "crates/simulation-model/src/lib.rs",
    """pub enum CommandPayload {
    NoOp { token: u64 },
}
""",
    """pub enum CommandPayload {
    NoOp { token: u64 },
    CreateCountryEntity,
    RemoveCountryEntity { country_id: CountryId },
    CreateCityEntity {
        country_id: Option<CountryId>,
        region_id: Option<RegionId>,
    },
    RemoveCityEntity { city_id: CityId },
    CreateRegionEntity { draft: RegionDraft },
    RemoveRegionEntity { region_id: RegionId },
}

impl CommandPayload {
    #[must_use]
    pub fn references_country(&self, country_id: CountryId) -> bool {
        match self {
            Self::RemoveCountryEntity { country_id: referenced } => *referenced == country_id,
            Self::CreateCityEntity { country_id: Some(referenced), .. } => *referenced == country_id,
            Self::CreateRegionEntity { draft } => {
                draft.political.legal_owner == Some(country_id)
                    || draft.political.controller == Some(country_id)
            }
            Self::NoOp { .. }
            | Self::CreateCountryEntity
            | Self::CreateCityEntity { country_id: None, .. }
            | Self::RemoveCityEntity { .. }
            | Self::RemoveRegionEntity { .. } => false,
        }
    }

    #[must_use]
    pub fn references_region(&self, region_id: RegionId) -> bool {
        match self {
            Self::CreateCityEntity { region_id: Some(referenced), .. } => *referenced == region_id,
            Self::CreateRegionEntity { draft } => draft.neighbors.contains(&region_id),
            Self::RemoveRegionEntity { region_id: referenced } => *referenced == region_id,
            Self::NoOp { .. }
            | Self::CreateCountryEntity
            | Self::RemoveCountryEntity { .. }
            | Self::CreateCityEntity { region_id: None, .. }
            | Self::RemoveCityEntity { .. } => false,
        }
    }
}
""",
)
replace_once(
    "crates/simulation-model/src/lib.rs",
    """impl CommandRequest {
    #[must_use]
    pub const fn user(timing: CommandTiming, payload: CommandPayload) -> Self {
""",
    """impl CommandRequest {
    #[must_use]
    pub const fn system(timing: CommandTiming, payload: CommandPayload) -> Self {
        Self {
            source: CommandSource::System,
            timing,
            payload,
        }
    }

    #[must_use]
    pub const fn user(timing: CommandTiming, payload: CommandPayload) -> Self {
""",
)

# Core wiring.
replace_once("crates/simulation-core/src/lib.rs", "mod event_ledger;\n", "mod entity_lifecycle;\nmod event_ledger;\n")
replace_once(
    "crates/simulation-core/src/lib.rs",
    """    /// Submits a user-originated command using the user intervention priority.
""",
    """    /// Submits a system-originated deterministic lifecycle command.
    ///
    /// # Errors
    /// Returns [`CommandError`] under the same scheduling rules as other commands.
    pub fn submit_system_command(
        &mut self,
        timing: CommandTiming,
        payload: CommandPayload,
    ) -> Result<CommandId, CommandError> {
        self.submit_command(CommandRequest::system(timing, payload))
    }

    /// Submits a user-originated command using the user intervention priority.
""",
)
replace_once(
    "crates/simulation-core/src/lib.rs",
    """        match &command.payload {
            CommandPayload::NoOp { .. } => {}
        }

        self.executed_commands.push(CommandExecutionRecord {
""",
    """        let lifecycle_event = self.apply_entity_payload(&command.payload).ok().flatten();

        self.executed_commands.push(CommandExecutionRecord {
""",
)
replace_once(
    "crates/simulation-core/src/lib.rs",
    """        let (category, source) = match command_source {
            CommandSource::User => (EventCategory::UserIntervention, EventSource::User),
            CommandSource::CountryAi(country_id) => {
                (EventCategory::System, EventSource::Country(country_id))
            }
        };

        self.event_ledger.append(
""",
    """        if let Some(payload) = lifecycle_event {
            self.event_ledger.append(
                executed_on,
                self.state.elapsed_days,
                EventDraft {
                    category: EventCategory::EntityLifecycle,
                    source: EventSource::System,
                    payload,
                },
            );
        }

        let (category, source) = match command_source {
            CommandSource::System => (EventCategory::System, EventSource::System),
            CommandSource::User => (EventCategory::UserIntervention, EventSource::User),
            CommandSource::CountryAi(country_id) => {
                (EventCategory::System, EventSource::Country(country_id))
            }
        };

        self.event_ledger.append(
""",
)

# Save format and aggregate reference validation.
replace_once(
    "crates/simulation-save/src/lib.rs",
    "/// Stage 2.4 adds finite resource stocks to authoritative state.\npub const SAVE_FORMAT_VERSION: u32 = 4;",
    "/// Stage 2.5 adds dynamic entity registries and stable allocator state.\npub const SAVE_FORMAT_VERSION: u32 = 5;",
)
replace_once(
    "crates/simulation-save/src/lib.rs",
    """    if world.spatial.validate().is_err() {
        return Err(SaveError::SnapshotInvariant(
            "world spatial state violates Stage 2.1 topology invariants",
        ));
    }
    Ok(())
}
""",
    """    if world.spatial.validate().is_err() {
        return Err(SaveError::SnapshotInvariant(
            "world spatial state violates Stage 2.1 topology invariants",
        ));
    }

    use simulation_model::EntityRegistry as _;
    for region in world.spatial.regions.iter() {
        for country in [region.political.legal_owner, region.political.controller]
            .into_iter()
            .flatten()
        {
            if !world.entities.countries.contains(country) {
                return Err(SaveError::SnapshotInvariant(
                    "Region references an unknown active country",
                ));
            }
        }
    }
    for city in world.entities.cities.iter() {
        if city
            .country
            .is_some_and(|country| !world.entities.countries.contains(country))
            || city
                .region
                .is_some_and(|region| world.spatial.region(region).is_none())
        {
            return Err(SaveError::SnapshotInvariant(
                "City references an unknown active entity",
            ));
        }
    }
    Ok(())
}
""",
)
replace_once(
    "crates/simulation-save/src/lib.rs",
    """                validate_command_event_attribution(event, *source)?;
            }
        }
    }
""",
    """                validate_command_event_attribution(event, *source)?;
            }
            EventPayload::EntityCreated { entity } | EventPayload::EntityRemoved { entity } => {
                if event.category != EventCategory::EntityLifecycle
                    || event.source != EventSource::System
                    || !entity.id.is_valid()
                {
                    return Err(SaveError::SnapshotInvariant(
                        "entity lifecycle event attribution is invalid",
                    ));
                }
            }
        }
    }
""",
)

# Binary imports.
replace_once(
    "crates/simulation-save/src/binary.rs",
    """use simulation_model::{
    BiomeClass, CommandExecutionRecord, CommandId, CommandPayload, CommandPriority, CommandSource,
    CountryId, EventCategory, EventId, EventPayload, EventRecord, EventSource, MapPoint,
    QueuedCommand, RegionId, RegionPoliticalState, RegionState, RegionSurface, ReliefClass,
    ResourceCellState, ResourceDeposit, ResourceFieldState, SimulationDate, TerrainSample,
    TerrainState, WorldBounds, WorldSpatialState, WorldState,
};
""",
    """use simulation_model::{
    BiomeClass, CityEntity, CityId, CityRegistry, CommandExecutionRecord, CommandId, CommandPayload,
    CommandPriority, CommandSource, CountryEntity, CountryId, CountryRegistry, EntityKind, EntityRef,
    EntityRegistry, EntityWorldState, EventCategory, EventId, EventPayload, EventRecord, EventSource,
    MapPoint, QueuedCommand, RegionDraft, RegionId, RegionPoliticalState, RegionRegistry, RegionState,
    RegionSurface, ReliefClass, ResourceCellState, ResourceDeposit, ResourceFieldState,
    SimulationDate, StableEntityId, TerrainSample, TerrainState, WorldBounds, WorldSpatialState,
    WorldState,
};
""",
)

# Binary world registry state.
replace_once(
    "crates/simulation-save/src/binary.rs",
    """            write_i32(bytes, bounds.min_z_m);
            write_i32(bytes, bounds.max_z_m);
            write_count(bytes, "regions", world.spatial.regions.len())?;
            for region in &world.spatial.regions {
                write_region(bytes, region)?;
            }
        }
    }

    Ok(())
}
""",
    """            write_i32(bytes, bounds.min_z_m);
            write_i32(bytes, bounds.max_z_m);
            write_u64(bytes, world.spatial.regions.next_id());
            write_count(bytes, "regions", world.spatial.regions.len())?;
            for region in &world.spatial.regions {
                write_region(bytes, region)?;
            }
        }
    }

    write_u64(bytes, world.entities.countries.next_id());
    write_count(bytes, "countries", world.entities.countries.len())?;
    for country in world.entities.countries.iter() {
        write_u64(bytes, country.id.0);
    }

    write_u64(bytes, world.entities.cities.next_id());
    write_count(bytes, "cities", world.entities.cities.len())?;
    for city in world.entities.cities.iter() {
        write_u64(bytes, city.id.0);
        write_optional_country(bytes, city.country);
        write_optional_region(bytes, city.region);
    }

    Ok(())
}
""",
)
replace_once(
    "crates/simulation-save/src/binary.rs",
    """            .map_err(|_| SaveError::InvalidBinary("invalid world bounds"))?;
            let region_count = cursor.read_count("regions")?;
            let mut regions = Vec::with_capacity(region_count);
            for _ in 0..region_count {
                regions.push(read_region(cursor)?);
            }
            WorldSpatialState::new_trial(bounds, regions)
                .map_err(|_| SaveError::InvalidBinary("invalid world spatial state"))?
        }
""",
    """            .map_err(|_| SaveError::InvalidBinary("invalid world bounds"))?;
            let region_next_id = cursor.read_u64()?;
            let region_count = cursor.read_count("regions")?;
            let mut regions = Vec::with_capacity(region_count);
            for _ in 0..region_count {
                regions.push(read_region(cursor)?);
            }
            let registry = RegionRegistry::from_parts(region_next_id, regions)
                .map_err(|_| SaveError::InvalidBinary("invalid Region registry"))?;
            WorldSpatialState::from_registry(bounds, registry)
                .map_err(|_| SaveError::InvalidBinary("invalid world spatial state"))?
        }
""",
)
replace_once(
    "crates/simulation-save/src/binary.rs",
    """    Ok(WorldState {
        date,
        elapsed_days,
        seed,
        terrain,
        resources,
        spatial,
    })
}
""",
    """    let country_next_id = cursor.read_u64()?;
    let country_count = cursor.read_count("countries")?;
    let mut countries = Vec::with_capacity(country_count);
    for _ in 0..country_count {
        countries.push(CountryEntity {
            id: CountryId(cursor.read_u64()?),
        });
    }
    let countries = CountryRegistry::from_parts(country_next_id, countries)
        .map_err(|_| SaveError::InvalidBinary("invalid Country registry"))?;

    let city_next_id = cursor.read_u64()?;
    let city_count = cursor.read_count("cities")?;
    let mut cities = Vec::with_capacity(city_count);
    for _ in 0..city_count {
        cities.push(CityEntity {
            id: CityId(cursor.read_u64()?),
            country: read_optional_country(cursor)?,
            region: read_optional_region(cursor)?,
        });
    }
    let cities = CityRegistry::from_parts(city_next_id, cities)
        .map_err(|_| SaveError::InvalidBinary("invalid City registry"))?;

    Ok(WorldState {
        date,
        elapsed_days,
        seed,
        terrain,
        resources,
        spatial,
        entities: EntityWorldState { countries, cities },
    })
}
""",
)

# Binary Region and ID widths.
replace_once(
    "crates/simulation-save/src/binary.rs",
    """fn write_region(bytes: &mut Vec<u8>, region: &RegionState) -> Result<(), SaveError> {
    write_u16(bytes, region.id.0);
""",
    """fn write_region(bytes: &mut Vec<u8>, region: &RegionState) -> Result<(), SaveError> {
    write_u64(bytes, region.id.0);
""",
)
replace_once(
    "crates/simulation-save/src/binary.rs",
    """    for &neighbor in &region.neighbors {
        write_u16(bytes, neighbor.0);
    }
""",
    """    for &neighbor in &region.neighbors {
        write_u64(bytes, neighbor.0);
    }
""",
)
replace_once(
    "crates/simulation-save/src/binary.rs",
    "let id = RegionId(cursor.read_u16()?);",
    "let id = RegionId(cursor.read_u64()?);",
)
replace_once(
    "crates/simulation-save/src/binary.rs",
    "neighbors.push(RegionId(cursor.read_u16()?));",
    "neighbors.push(RegionId(cursor.read_u64()?));",
)
replace_once(
    "crates/simulation-save/src/binary.rs",
    "write_u16(bytes, country.0);",
    "write_u64(bytes, country.0);",
)
replace_once(
    "crates/simulation-save/src/binary.rs",
    "1 => Ok(Some(CountryId(cursor.read_u16()?))),",
    "1 => Ok(Some(CountryId(cursor.read_u64()?))),",
)

# Optional Region and RegionDraft codec.
anchor = """fn read_optional_country(cursor: &mut Cursor<'_>) -> Result<Option<CountryId>, SaveError> {
    match cursor.read_u8()? {
        0 => Ok(None),
        1 => Ok(Some(CountryId(cursor.read_u64()?))),
        tag => Err(SaveError::InvalidTag {
            field: "optional country",
            tag,
        }),
    }
}
"""
insert = anchor + """
fn write_optional_region(bytes: &mut Vec<u8>, region: Option<RegionId>) {
    match region {
        None => write_u8(bytes, 0),
        Some(region) => {
            write_u8(bytes, 1);
            write_u64(bytes, region.0);
        }
    }
}

fn read_optional_region(cursor: &mut Cursor<'_>) -> Result<Option<RegionId>, SaveError> {
    match cursor.read_u8()? {
        0 => Ok(None),
        1 => Ok(Some(RegionId(cursor.read_u64()?))),
        tag => Err(SaveError::InvalidTag {
            field: "optional Region",
            tag,
        }),
    }
}

fn write_region_draft(bytes: &mut Vec<u8>, draft: &RegionDraft) -> Result<(), SaveError> {
    write_region_surface(bytes, draft.surface);
    write_point(bytes, draft.center);
    write_count(bytes, "draft_boundary_points", draft.boundary.len())?;
    for &point in &draft.boundary {
        write_point(bytes, point);
    }
    write_count(bytes, "draft_neighbors", draft.neighbors.len())?;
    for &neighbor in &draft.neighbors {
        write_u64(bytes, neighbor.0);
    }
    write_optional_country(bytes, draft.political.legal_owner);
    write_optional_country(bytes, draft.political.controller);
    Ok(())
}

fn read_region_draft(cursor: &mut Cursor<'_>) -> Result<RegionDraft, SaveError> {
    let surface = read_region_surface(cursor)?;
    let center = read_point(cursor)?;
    let boundary_count = cursor.read_count("draft_boundary_points")?;
    let mut boundary = Vec::with_capacity(boundary_count);
    for _ in 0..boundary_count {
        boundary.push(read_point(cursor)?);
    }
    let neighbor_count = cursor.read_count("draft_neighbors")?;
    let mut neighbors = Vec::with_capacity(neighbor_count);
    for _ in 0..neighbor_count {
        neighbors.push(RegionId(cursor.read_u64()?));
    }
    Ok(RegionDraft {
        surface,
        center,
        boundary,
        neighbors,
        political: RegionPoliticalState {
            legal_owner: read_optional_country(cursor)?,
            controller: read_optional_country(cursor)?,
        },
    })
}
"""
replace_once("crates/simulation-save/src/binary.rs", anchor, insert)

# Command source and priority codec.
replace_once(
    "crates/simulation-save/src/binary.rs",
    """    match source {
        CommandSource::User => write_u8(bytes, 0),
        CommandSource::CountryAi(country_id) => {
            write_u8(bytes, 1);
            write_u16(bytes, country_id.0);
        }
    }
""",
    """    match source {
        CommandSource::System => write_u8(bytes, 0),
        CommandSource::User => write_u8(bytes, 1),
        CommandSource::CountryAi(country_id) => {
            write_u8(bytes, 2);
            write_u64(bytes, country_id.0);
        }
    }
""",
)
replace_once(
    "crates/simulation-save/src/binary.rs",
    """    match cursor.read_u8()? {
        0 => Ok(CommandSource::User),
        1 => Ok(CommandSource::CountryAi(CountryId(cursor.read_u16()?))),
""",
    """    match cursor.read_u8()? {
        0 => Ok(CommandSource::System),
        1 => Ok(CommandSource::User),
        2 => Ok(CommandSource::CountryAi(CountryId(cursor.read_u64()?))),
""",
)
replace_once(
    "crates/simulation-save/src/binary.rs",
    """    let tag = match priority {
        CommandPriority::UserIntervention => 10,
        CommandPriority::CountryAi => 20,
    };
""",
    """    let tag = match priority {
        CommandPriority::System => 0,
        CommandPriority::UserIntervention => 10,
        CommandPriority::CountryAi => 20,
    };
""",
)
replace_once(
    "crates/simulation-save/src/binary.rs",
    """    match cursor.read_u8()? {
        10 => Ok(CommandPriority::UserIntervention),
""",
    """    match cursor.read_u8()? {
        0 => Ok(CommandPriority::System),
        10 => Ok(CommandPriority::UserIntervention),
""",
)

# Command payload codec.
replace_once(
    "crates/simulation-save/src/binary.rs",
    """        CommandPayload::NoOp { token } => {
            write_u8(bytes, 0);
            write_u64(bytes, *token);
        }
    }
}
""",
    """        CommandPayload::NoOp { token } => {
            write_u8(bytes, 0);
            write_u64(bytes, *token);
        }
        CommandPayload::CreateCountryEntity => write_u8(bytes, 1),
        CommandPayload::RemoveCountryEntity { country_id } => {
            write_u8(bytes, 2);
            write_u64(bytes, country_id.0);
        }
        CommandPayload::CreateCityEntity { country_id, region_id } => {
            write_u8(bytes, 3);
            write_optional_country(bytes, *country_id);
            write_optional_region(bytes, *region_id);
        }
        CommandPayload::RemoveCityEntity { city_id } => {
            write_u8(bytes, 4);
            write_u64(bytes, city_id.0);
        }
        CommandPayload::CreateRegionEntity { draft } => {
            write_u8(bytes, 5);
            let _ = write_region_draft(bytes, draft);
        }
        CommandPayload::RemoveRegionEntity { region_id } => {
            write_u8(bytes, 6);
            write_u64(bytes, region_id.0);
        }
    }
}
""",
)
replace_once(
    "crates/simulation-save/src/binary.rs",
    """        0 => Ok(CommandPayload::NoOp {
            token: cursor.read_u64()?,
        }),
        tag => Err(SaveError::InvalidTag {
""",
    """        0 => Ok(CommandPayload::NoOp {
            token: cursor.read_u64()?,
        }),
        1 => Ok(CommandPayload::CreateCountryEntity),
        2 => Ok(CommandPayload::RemoveCountryEntity {
            country_id: CountryId(cursor.read_u64()?),
        }),
        3 => Ok(CommandPayload::CreateCityEntity {
            country_id: read_optional_country(cursor)?,
            region_id: read_optional_region(cursor)?,
        }),
        4 => Ok(CommandPayload::RemoveCityEntity {
            city_id: CityId(cursor.read_u64()?),
        }),
        5 => Ok(CommandPayload::CreateRegionEntity {
            draft: read_region_draft(cursor)?,
        }),
        6 => Ok(CommandPayload::RemoveRegionEntity {
            region_id: RegionId(cursor.read_u64()?),
        }),
        tag => Err(SaveError::InvalidTag {
""",
)

# Event codec.
replace_once(
    "crates/simulation-save/src/binary.rs",
    """        EventCategory::UserIntervention => 8,
    };
""",
    """        EventCategory::UserIntervention => 8,
        EventCategory::EntityLifecycle => 9,
    };
""",
)
replace_once(
    "crates/simulation-save/src/binary.rs",
    """        8 => Ok(EventCategory::UserIntervention),
        tag => Err(SaveError::InvalidTag {
""",
    """        8 => Ok(EventCategory::UserIntervention),
        9 => Ok(EventCategory::EntityLifecycle),
        tag => Err(SaveError::InvalidTag {
""",
)
replace_once(
    "crates/simulation-save/src/binary.rs",
    "write_u16(bytes, country_id.0);",
    "write_u64(bytes, country_id.0);",
)
replace_once(
    "crates/simulation-save/src/binary.rs",
    "2 => Ok(EventSource::Country(CountryId(cursor.read_u16()?))),",
    "2 => Ok(EventSource::Country(CountryId(cursor.read_u64()?))),",
)
replace_once(
    "crates/simulation-save/src/binary.rs",
    """        EventPayload::CommandExecuted { command_id } => {
            write_u8(bytes, 0);
            write_u64(bytes, command_id.0);
        }
    }
}
""",
    """        EventPayload::CommandExecuted { command_id } => {
            write_u8(bytes, 0);
            write_u64(bytes, command_id.0);
        }
        EventPayload::EntityCreated { entity } => {
            write_u8(bytes, 1);
            write_entity_ref(bytes, entity);
        }
        EventPayload::EntityRemoved { entity } => {
            write_u8(bytes, 2);
            write_entity_ref(bytes, entity);
        }
    }
}

fn write_entity_ref(bytes: &mut Vec<u8>, entity: EntityRef) {
    write_u8(
        bytes,
        match entity.kind {
            EntityKind::Country => 0,
            EntityKind::Region => 1,
            EntityKind::City => 2,
        },
    );
    write_u64(bytes, entity.id.0);
}

fn read_entity_ref(cursor: &mut Cursor<'_>) -> Result<EntityRef, SaveError> {
    let kind = match cursor.read_u8()? {
        0 => EntityKind::Country,
        1 => EntityKind::Region,
        2 => EntityKind::City,
        tag => return Err(SaveError::InvalidTag { field: "entity kind", tag }),
    };
    let id = StableEntityId(cursor.read_u64()?);
    if !id.is_valid() {
        return Err(SaveError::InvalidBinary("invalid entity reference id"));
    }
    Ok(EntityRef { kind, id })
}
""",
)
replace_once(
    "crates/simulation-save/src/binary.rs",
    """        0 => Ok(EventPayload::CommandExecuted {
            command_id: CommandId(cursor.read_u64()?),
        }),
        tag => Err(SaveError::InvalidTag {
""",
    """        0 => Ok(EventPayload::CommandExecuted {
            command_id: CommandId(cursor.read_u64()?),
        }),
        1 => Ok(EventPayload::EntityCreated {
            entity: read_entity_ref(cursor)?,
        }),
        2 => Ok(EventPayload::EntityRemoved {
            entity: read_entity_ref(cursor)?,
        }),
        tag => Err(SaveError::InvalidTag {
""",
)

# Render protocol and TS use string IDs at the JS boundary.
replace_once("crates/simulation-protocol/src/lib.rs", "pub const RENDER_SNAPSHOT_VERSION: u32 = 3;", "pub const RENDER_SNAPSHOT_VERSION: u32 = 4;")
replace_once(
    "crates/simulation-protocol/src/lib.rs",
    """pub struct RenderRegionSnapshot {
    pub id: u16,
    pub surface: RenderRegionSurface,
    pub center: RenderMapPoint,
    pub boundary: Vec<RenderMapPoint>,
    pub neighbors: Vec<u16>,
    pub legal_owner: Option<u16>,
    pub controller: Option<u16>,
}
""",
    """pub struct RenderRegionSnapshot {
    pub id: String,
    pub surface: RenderRegionSurface,
    pub center: RenderMapPoint,
    pub boundary: Vec<RenderMapPoint>,
    pub neighbors: Vec<String>,
    pub legal_owner: Option<String>,
    pub controller: Option<String>,
}
""",
)
replace_once(
    "crates/simulation-protocol/src/lib.rs",
    """            id: region.id.0,
            surface: region.surface.into(),
            center: region.center.into(),
            boundary: region.boundary.iter().copied().map(Into::into).collect(),
            neighbors: region.neighbors.iter().map(|neighbor| neighbor.0).collect(),
            legal_owner: region.political.legal_owner.map(|country| country.0),
            controller: region.political.controller.map(|country| country.0),
""",
    """            id: region.id.0.to_string(),
            surface: region.surface.into(),
            center: region.center.into(),
            boundary: region.boundary.iter().copied().map(Into::into).collect(),
            neighbors: region
                .neighbors
                .iter()
                .map(|neighbor| neighbor.0.to_string())
                .collect(),
            legal_owner: region
                .political
                .legal_owner
                .map(|country| country.0.to_string()),
            controller: region
                .political
                .controller
                .map(|country| country.0.to_string()),
""",
)
replace_once(
    "viewer/src/types.ts",
    """export interface RenderRegionSnapshot {
  id: number;
  surface: RegionSurface;
  center: RenderMapPoint;
  boundary: RenderMapPoint[];
  neighbors: number[];
  legalOwner: number | null;
  controller: number | null;
}
""",
    """export interface RenderRegionSnapshot {
  id: string;
  surface: RegionSurface;
  center: RenderMapPoint;
  boundary: RenderMapPoint[];
  neighbors: string[];
  legalOwner: string | null;
  controller: string | null;
}
""",
)

print("Stage 2.5 migration patches applied")
