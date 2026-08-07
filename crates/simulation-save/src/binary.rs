use simulation_model::{
    BiomeClass, CityEntity, CityId, CityRegistry, CommandExecutionRecord, CommandId,
    CommandPayload, CommandPriority, CommandSource, CountryEntity, CountryId, CountryRegistry,
    EntityKind, EntityRef, EntityRegistry, EntityWorldState, EventCategory, EventId, EventPayload,
    EventRecord, EventSource, MapPoint, QueuedCommand, RegionDraft, RegionId, RegionPoliticalState,
    RegionRegistry, RegionState, RegionSurface, ReliefClass, ResourceCellState, ResourceDeposit,
    ResourceFieldState, SimulationDate, StableEntityId, TerrainSample, TerrainState, WorldBounds,
    WorldSpatialState, WorldState,
};

use crate::{EngineSnapshot, SaveError, SAVE_FORMAT_VERSION};

const STATE_MAGIC: [u8; 8] = *b"GAONST01";
const MAX_RECORD_COUNT: u64 = 10_000_000;

pub(crate) fn encode_snapshot(snapshot: &EngineSnapshot) -> Result<Vec<u8>, SaveError> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&STATE_MAGIC);
    write_u32(&mut bytes, SAVE_FORMAT_VERSION);
    write_world(&mut bytes, &snapshot.world)?;
    write_u64(&mut bytes, snapshot.next_command_id);
    write_u64(&mut bytes, snapshot.next_event_id);

    write_count(
        &mut bytes,
        "pending_commands",
        snapshot.pending_commands.len(),
    )?;
    for command in &snapshot.pending_commands {
        write_queued_command(&mut bytes, command);
    }

    write_count(
        &mut bytes,
        "executed_commands",
        snapshot.executed_commands.len(),
    )?;
    for record in &snapshot.executed_commands {
        write_execution_record(&mut bytes, record);
    }

    write_count(&mut bytes, "events", snapshot.events.len())?;
    for event in &snapshot.events {
        write_event_record(&mut bytes, event);
    }

    Ok(bytes)
}

pub(crate) fn decode_snapshot(bytes: &[u8]) -> Result<EngineSnapshot, SaveError> {
    let mut cursor = Cursor::new(bytes);
    let magic = cursor.read_array::<8>()?;
    if magic != STATE_MAGIC {
        return Err(SaveError::InvalidBinary("invalid state magic"));
    }

    let version = cursor.read_u32()?;
    if version != SAVE_FORMAT_VERSION {
        return Err(SaveError::UnsupportedBinaryVersion { found: version });
    }

    let world = read_world(&mut cursor)?;
    let next_command_id = cursor.read_u64()?;
    let next_event_id = cursor.read_u64()?;

    let pending_count = cursor.read_count("pending_commands")?;
    let mut pending_commands = Vec::with_capacity(pending_count);
    for _ in 0..pending_count {
        pending_commands.push(read_queued_command(&mut cursor)?);
    }

    let executed_count = cursor.read_count("executed_commands")?;
    let mut executed_commands = Vec::with_capacity(executed_count);
    for _ in 0..executed_count {
        executed_commands.push(read_execution_record(&mut cursor)?);
    }

    let event_count = cursor.read_count("events")?;
    let mut events = Vec::with_capacity(event_count);
    for _ in 0..event_count {
        events.push(read_event_record(&mut cursor)?);
    }

    if !cursor.is_finished() {
        return Err(SaveError::TrailingBinaryData);
    }

    Ok(EngineSnapshot {
        world,
        pending_commands,
        executed_commands,
        events,
        next_command_id,
        next_event_id,
    })
}

fn write_world(bytes: &mut Vec<u8>, world: &WorldState) -> Result<(), SaveError> {
    write_date(bytes, world.date);
    write_u64(bytes, world.elapsed_days);
    write_u64(bytes, world.seed);

    match &world.terrain {
        None => write_u8(bytes, 0),
        Some(terrain) => {
            write_u8(bytes, 1);
            write_terrain(bytes, terrain)?;
        }
    }

    match &world.resources {
        None => write_u8(bytes, 0),
        Some(resources) => {
            write_u8(bytes, 1);
            write_resource_field(bytes, resources)?;
        }
    }

    match world.spatial.bounds {
        None => write_u8(bytes, 0),
        Some(bounds) => {
            write_u8(bytes, 1);
            write_i32(bytes, bounds.min_x_m);
            write_i32(bytes, bounds.max_x_m);
            write_i32(bytes, bounds.min_z_m);
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

fn read_world(cursor: &mut Cursor<'_>) -> Result<WorldState, SaveError> {
    let date = read_date(cursor)?;
    let elapsed_days = cursor.read_u64()?;
    let seed = cursor.read_u64()?;

    let terrain = match cursor.read_u8()? {
        0 => None,
        1 => Some(read_terrain(cursor)?),
        tag => {
            return Err(SaveError::InvalidTag {
                field: "terrain state",
                tag,
            });
        }
    };

    let resources = match cursor.read_u8()? {
        0 => None,
        1 => Some(read_resource_field(cursor)?),
        tag => {
            return Err(SaveError::InvalidTag {
                field: "resource state",
                tag,
            });
        }
    };

    let spatial = match cursor.read_u8()? {
        0 => WorldSpatialState::uninitialized(),
        1 => {
            let bounds = WorldBounds::new(
                cursor.read_i32()?,
                cursor.read_i32()?,
                cursor.read_i32()?,
                cursor.read_i32()?,
            )
            .map_err(|_| SaveError::InvalidBinary("invalid world bounds"))?;
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
        tag => {
            return Err(SaveError::InvalidTag {
                field: "world spatial state",
                tag,
            });
        }
    };

    let country_next_id = cursor.read_u64()?;
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

fn write_terrain(bytes: &mut Vec<u8>, terrain: &TerrainState) -> Result<(), SaveError> {
    write_i32(bytes, terrain.bounds.min_x_m);
    write_i32(bytes, terrain.bounds.max_x_m);
    write_i32(bytes, terrain.bounds.min_z_m);
    write_i32(bytes, terrain.bounds.max_z_m);
    write_u16(bytes, terrain.width);
    write_u16(bytes, terrain.height);
    write_i32(bytes, terrain.spacing_m);
    write_i16(bytes, terrain.sea_level_m);
    write_count(bytes, "terrain_samples", terrain.samples.len())?;
    for sample in &terrain.samples {
        write_i16(bytes, sample.elevation_m);
        write_u16(bytes, sample.moisture_permille);
        write_relief(bytes, sample.relief);
        write_biome(bytes, sample.biome);
    }
    Ok(())
}

fn read_terrain(cursor: &mut Cursor<'_>) -> Result<TerrainState, SaveError> {
    let bounds = WorldBounds::new(
        cursor.read_i32()?,
        cursor.read_i32()?,
        cursor.read_i32()?,
        cursor.read_i32()?,
    )
    .map_err(|_| SaveError::InvalidBinary("invalid terrain bounds"))?;
    let width = cursor.read_u16()?;
    let height = cursor.read_u16()?;
    let spacing_m = cursor.read_i32()?;
    let sea_level_m = cursor.read_i16()?;
    let count = cursor.read_count("terrain_samples")?;
    let mut samples = Vec::with_capacity(count);
    for _ in 0..count {
        samples.push(TerrainSample {
            elevation_m: cursor.read_i16()?,
            moisture_permille: cursor.read_u16()?,
            relief: read_relief(cursor)?,
            biome: read_biome(cursor)?,
        });
    }

    let terrain = TerrainState {
        bounds,
        width,
        height,
        spacing_m,
        sea_level_m,
        samples,
    };
    terrain
        .validate()
        .map_err(|_| SaveError::InvalidBinary("invalid terrain state"))?;
    Ok(terrain)
}

fn write_resource_field(
    bytes: &mut Vec<u8>,
    resources: &ResourceFieldState,
) -> Result<(), SaveError> {
    write_count(bytes, "resource_cells", resources.cells.len())?;
    for cell in &resources.cells {
        write_u32(bytes, cell.food_capacity_tonnes_per_year);
        write_resource_deposit(bytes, cell.energy);
        write_resource_deposit(bytes, cell.metals);
        write_resource_deposit(bytes, cell.construction);
    }
    Ok(())
}

fn read_resource_field(cursor: &mut Cursor<'_>) -> Result<ResourceFieldState, SaveError> {
    let count = cursor.read_count("resource_cells")?;
    let mut cells = Vec::with_capacity(count);
    for _ in 0..count {
        cells.push(ResourceCellState {
            food_capacity_tonnes_per_year: cursor.read_u32()?,
            energy: read_resource_deposit(cursor)?,
            metals: read_resource_deposit(cursor)?,
            construction: read_resource_deposit(cursor)?,
        });
    }
    ResourceFieldState::new(cells)
        .map_err(|_| SaveError::InvalidBinary("invalid Stage 2.4 resource field"))
}

fn write_resource_deposit(bytes: &mut Vec<u8>, deposit: ResourceDeposit) {
    write_u64(bytes, deposit.initial_quantity);
    write_u64(bytes, deposit.remaining_quantity);
    write_u16(bytes, deposit.quality_permille);
    write_u16(bytes, deposit.accessibility_permille);
}

fn read_resource_deposit(cursor: &mut Cursor<'_>) -> Result<ResourceDeposit, SaveError> {
    let deposit = ResourceDeposit {
        initial_quantity: cursor.read_u64()?,
        remaining_quantity: cursor.read_u64()?,
        quality_permille: cursor.read_u16()?,
        accessibility_permille: cursor.read_u16()?,
    };
    deposit
        .validate()
        .map_err(|_| SaveError::InvalidBinary("invalid resource deposit"))?;
    Ok(deposit)
}

fn write_relief(bytes: &mut Vec<u8>, relief: ReliefClass) {
    let tag = match relief {
        ReliefClass::DeepOcean => 0,
        ReliefClass::ShallowOcean => 1,
        ReliefClass::Coast => 2,
        ReliefClass::Plains => 3,
        ReliefClass::Hills => 4,
        ReliefClass::Mountains => 5,
    };
    write_u8(bytes, tag);
}

fn read_relief(cursor: &mut Cursor<'_>) -> Result<ReliefClass, SaveError> {
    match cursor.read_u8()? {
        0 => Ok(ReliefClass::DeepOcean),
        1 => Ok(ReliefClass::ShallowOcean),
        2 => Ok(ReliefClass::Coast),
        3 => Ok(ReliefClass::Plains),
        4 => Ok(ReliefClass::Hills),
        5 => Ok(ReliefClass::Mountains),
        tag => Err(SaveError::InvalidTag {
            field: "terrain relief",
            tag,
        }),
    }
}

fn write_biome(bytes: &mut Vec<u8>, biome: BiomeClass) {
    let tag = match biome {
        BiomeClass::Ocean => 0,
        BiomeClass::Grassland => 1,
        BiomeClass::Forest => 2,
        BiomeClass::Desert => 3,
        BiomeClass::Wetland => 4,
        BiomeClass::Alpine => 5,
    };
    write_u8(bytes, tag);
}

fn read_biome(cursor: &mut Cursor<'_>) -> Result<BiomeClass, SaveError> {
    match cursor.read_u8()? {
        0 => Ok(BiomeClass::Ocean),
        1 => Ok(BiomeClass::Grassland),
        2 => Ok(BiomeClass::Forest),
        3 => Ok(BiomeClass::Desert),
        4 => Ok(BiomeClass::Wetland),
        5 => Ok(BiomeClass::Alpine),
        tag => Err(SaveError::InvalidTag {
            field: "terrain biome",
            tag,
        }),
    }
}

fn write_region(bytes: &mut Vec<u8>, region: &RegionState) -> Result<(), SaveError> {
    write_u64(bytes, region.id.0);
    write_region_surface(bytes, region.surface);
    write_point(bytes, region.center);

    write_count(bytes, "region_boundary_points", region.boundary.len())?;
    for &point in &region.boundary {
        write_point(bytes, point);
    }

    write_count(bytes, "region_neighbors", region.neighbors.len())?;
    for &neighbor in &region.neighbors {
        write_u64(bytes, neighbor.0);
    }

    write_optional_country(bytes, region.political.legal_owner);
    write_optional_country(bytes, region.political.controller);
    Ok(())
}

fn read_region(cursor: &mut Cursor<'_>) -> Result<RegionState, SaveError> {
    let id = RegionId(cursor.read_u64()?);
    let surface = read_region_surface(cursor)?;
    let center = read_point(cursor)?;

    let boundary_count = cursor.read_count("region_boundary_points")?;
    let mut boundary = Vec::with_capacity(boundary_count);
    for _ in 0..boundary_count {
        boundary.push(read_point(cursor)?);
    }

    let neighbor_count = cursor.read_count("region_neighbors")?;
    let mut neighbors = Vec::with_capacity(neighbor_count);
    for _ in 0..neighbor_count {
        neighbors.push(RegionId(cursor.read_u64()?));
    }

    let legal_owner = read_optional_country(cursor)?;
    let controller = read_optional_country(cursor)?;

    Ok(RegionState {
        id,
        surface,
        center,
        boundary,
        neighbors,
        political: RegionPoliticalState {
            legal_owner,
            controller,
        },
    })
}

fn write_region_surface(bytes: &mut Vec<u8>, surface: RegionSurface) {
    write_u8(
        bytes,
        match surface {
            RegionSurface::Land => 0,
            RegionSurface::Ocean => 1,
        },
    );
}

fn read_region_surface(cursor: &mut Cursor<'_>) -> Result<RegionSurface, SaveError> {
    match cursor.read_u8()? {
        0 => Ok(RegionSurface::Land),
        1 => Ok(RegionSurface::Ocean),
        tag => Err(SaveError::InvalidTag {
            field: "region surface",
            tag,
        }),
    }
}

fn write_point(bytes: &mut Vec<u8>, point: MapPoint) {
    write_i32(bytes, point.x_m);
    write_i32(bytes, point.z_m);
}

fn read_point(cursor: &mut Cursor<'_>) -> Result<MapPoint, SaveError> {
    Ok(MapPoint::new(cursor.read_i32()?, cursor.read_i32()?))
}

fn write_optional_country(bytes: &mut Vec<u8>, country: Option<CountryId>) {
    match country {
        None => write_u8(bytes, 0),
        Some(country) => {
            write_u8(bytes, 1);
            write_u64(bytes, country.0);
        }
    }
}

fn read_optional_country(cursor: &mut Cursor<'_>) -> Result<Option<CountryId>, SaveError> {
    match cursor.read_u8()? {
        0 => Ok(None),
        1 => Ok(Some(CountryId(cursor.read_u64()?))),
        tag => Err(SaveError::InvalidTag {
            field: "optional country",
            tag,
        }),
    }
}

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

fn write_queued_command(bytes: &mut Vec<u8>, command: &QueuedCommand) {
    write_u64(bytes, command.id.0);
    write_command_source(bytes, command.source);
    write_date(bytes, command.submitted_on);
    write_date(bytes, command.execute_on);
    write_command_priority(bytes, command.priority);
    write_command_payload(bytes, &command.payload);
}

fn read_queued_command(cursor: &mut Cursor<'_>) -> Result<QueuedCommand, SaveError> {
    Ok(QueuedCommand {
        id: CommandId(cursor.read_u64()?),
        source: read_command_source(cursor)?,
        submitted_on: read_date(cursor)?,
        execute_on: read_date(cursor)?,
        priority: read_command_priority(cursor)?,
        payload: read_command_payload(cursor)?,
    })
}

fn write_execution_record(bytes: &mut Vec<u8>, record: &CommandExecutionRecord) {
    write_queued_command(bytes, &record.command);
    write_date(bytes, record.executed_on);
}

fn read_execution_record(cursor: &mut Cursor<'_>) -> Result<CommandExecutionRecord, SaveError> {
    Ok(CommandExecutionRecord {
        command: read_queued_command(cursor)?,
        executed_on: read_date(cursor)?,
    })
}

fn write_event_record(bytes: &mut Vec<u8>, event: &EventRecord) {
    write_u64(bytes, event.id.0);
    write_date(bytes, event.occurred_on);
    write_u64(bytes, event.tick_index);
    write_event_category(bytes, event.category);
    write_event_source(bytes, event.source);
    write_event_payload(bytes, event.payload);
}

fn read_event_record(cursor: &mut Cursor<'_>) -> Result<EventRecord, SaveError> {
    Ok(EventRecord {
        id: EventId(cursor.read_u64()?),
        occurred_on: read_date(cursor)?,
        tick_index: cursor.read_u64()?,
        category: read_event_category(cursor)?,
        source: read_event_source(cursor)?,
        payload: read_event_payload(cursor)?,
    })
}

fn write_command_source(bytes: &mut Vec<u8>, source: CommandSource) {
    match source {
        CommandSource::System => write_u8(bytes, 0),
        CommandSource::User => write_u8(bytes, 1),
        CommandSource::CountryAi(country_id) => {
            write_u8(bytes, 2);
            write_u64(bytes, country_id.0);
        }
    }
}

fn read_command_source(cursor: &mut Cursor<'_>) -> Result<CommandSource, SaveError> {
    match cursor.read_u8()? {
        0 => Ok(CommandSource::System),
        1 => Ok(CommandSource::User),
        2 => Ok(CommandSource::CountryAi(CountryId(cursor.read_u64()?))),
        tag => Err(SaveError::InvalidTag {
            field: "command source",
            tag,
        }),
    }
}

fn write_command_priority(bytes: &mut Vec<u8>, priority: CommandPriority) {
    let tag = match priority {
        CommandPriority::System => 0,
        CommandPriority::UserIntervention => 10,
        CommandPriority::CountryAi => 20,
    };
    write_u8(bytes, tag);
}

fn read_command_priority(cursor: &mut Cursor<'_>) -> Result<CommandPriority, SaveError> {
    match cursor.read_u8()? {
        0 => Ok(CommandPriority::System),
        10 => Ok(CommandPriority::UserIntervention),
        20 => Ok(CommandPriority::CountryAi),
        tag => Err(SaveError::InvalidTag {
            field: "command priority",
            tag,
        }),
    }
}

fn write_command_payload(bytes: &mut Vec<u8>, payload: &CommandPayload) {
    match payload {
        CommandPayload::NoOp { token } => {
            write_u8(bytes, 0);
            write_u64(bytes, *token);
        }
        CommandPayload::CreateCountryEntity => write_u8(bytes, 1),
        CommandPayload::RemoveCountryEntity { country_id } => {
            write_u8(bytes, 2);
            write_u64(bytes, country_id.0);
        }
        CommandPayload::CreateCityEntity {
            country_id,
            region_id,
        } => {
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

fn read_command_payload(cursor: &mut Cursor<'_>) -> Result<CommandPayload, SaveError> {
    match cursor.read_u8()? {
        0 => Ok(CommandPayload::NoOp {
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
            field: "command payload",
            tag,
        }),
    }
}

fn write_event_category(bytes: &mut Vec<u8>, category: EventCategory) {
    let tag = match category {
        EventCategory::System => 0,
        EventCategory::Economic => 1,
        EventCategory::Policy => 2,
        EventCategory::Diplomatic => 3,
        EventCategory::Military => 4,
        EventCategory::War => 5,
        EventCategory::Occupation => 6,
        EventCategory::Treaty => 7,
        EventCategory::UserIntervention => 8,
        EventCategory::EntityLifecycle => 9,
    };
    write_u8(bytes, tag);
}

fn read_event_category(cursor: &mut Cursor<'_>) -> Result<EventCategory, SaveError> {
    match cursor.read_u8()? {
        0 => Ok(EventCategory::System),
        1 => Ok(EventCategory::Economic),
        2 => Ok(EventCategory::Policy),
        3 => Ok(EventCategory::Diplomatic),
        4 => Ok(EventCategory::Military),
        5 => Ok(EventCategory::War),
        6 => Ok(EventCategory::Occupation),
        7 => Ok(EventCategory::Treaty),
        8 => Ok(EventCategory::UserIntervention),
        9 => Ok(EventCategory::EntityLifecycle),
        tag => Err(SaveError::InvalidTag {
            field: "event category",
            tag,
        }),
    }
}

fn write_event_source(bytes: &mut Vec<u8>, source: EventSource) {
    match source {
        EventSource::System => write_u8(bytes, 0),
        EventSource::User => write_u8(bytes, 1),
        EventSource::Country(country_id) => {
            write_u8(bytes, 2);
            write_u64(bytes, country_id.0);
        }
    }
}

fn read_event_source(cursor: &mut Cursor<'_>) -> Result<EventSource, SaveError> {
    match cursor.read_u8()? {
        0 => Ok(EventSource::System),
        1 => Ok(EventSource::User),
        2 => Ok(EventSource::Country(CountryId(cursor.read_u64()?))),
        tag => Err(SaveError::InvalidTag {
            field: "event source",
            tag,
        }),
    }
}

fn write_event_payload(bytes: &mut Vec<u8>, payload: EventPayload) {
    match payload {
        EventPayload::CommandExecuted { command_id } => {
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
        tag => {
            return Err(SaveError::InvalidTag {
                field: "entity kind",
                tag,
            })
        }
    };
    let id = StableEntityId(cursor.read_u64()?);
    if !id.is_valid() {
        return Err(SaveError::InvalidBinary("invalid entity reference id"));
    }
    Ok(EntityRef { kind, id })
}

fn read_event_payload(cursor: &mut Cursor<'_>) -> Result<EventPayload, SaveError> {
    match cursor.read_u8()? {
        0 => Ok(EventPayload::CommandExecuted {
            command_id: CommandId(cursor.read_u64()?),
        }),
        1 => Ok(EventPayload::EntityCreated {
            entity: read_entity_ref(cursor)?,
        }),
        2 => Ok(EventPayload::EntityRemoved {
            entity: read_entity_ref(cursor)?,
        }),
        tag => Err(SaveError::InvalidTag {
            field: "event payload",
            tag,
        }),
    }
}

fn write_date(bytes: &mut Vec<u8>, date: SimulationDate) {
    write_u32(bytes, date.year);
    write_u8(bytes, date.month);
    write_u8(bytes, date.day);
}

fn read_date(cursor: &mut Cursor<'_>) -> Result<SimulationDate, SaveError> {
    let date = SimulationDate::new(cursor.read_u32()?, cursor.read_u8()?, cursor.read_u8()?);
    if !date.is_valid() {
        return Err(SaveError::InvalidBinary("invalid simulation date"));
    }
    Ok(date)
}

fn write_count(bytes: &mut Vec<u8>, section: &'static str, count: usize) -> Result<(), SaveError> {
    let count = u64::try_from(count).map_err(|_| SaveError::RecordCountOverflow { section })?;
    write_u64(bytes, count);
    Ok(())
}

fn write_u8(bytes: &mut Vec<u8>, value: u8) {
    bytes.push(value);
}

fn write_u16(bytes: &mut Vec<u8>, value: u16) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn write_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn write_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn write_i16(bytes: &mut Vec<u8>, value: i16) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn write_i32(bytes: &mut Vec<u8>, value: i32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

struct Cursor<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> Cursor<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    const fn is_finished(&self) -> bool {
        self.position == self.bytes.len()
    }

    fn read_count(&mut self, section: &'static str) -> Result<usize, SaveError> {
        let count = self.read_u64()?;
        if count > MAX_RECORD_COUNT {
            return Err(SaveError::RecordLimitExceeded { section, count });
        }
        usize::try_from(count).map_err(|_| SaveError::RecordLimitExceeded { section, count })
    }

    fn read_u8(&mut self) -> Result<u8, SaveError> {
        Ok(self.read_array::<1>()?[0])
    }

    fn read_u16(&mut self) -> Result<u16, SaveError> {
        Ok(u16::from_le_bytes(self.read_array::<2>()?))
    }

    fn read_u32(&mut self) -> Result<u32, SaveError> {
        Ok(u32::from_le_bytes(self.read_array::<4>()?))
    }

    fn read_u64(&mut self) -> Result<u64, SaveError> {
        Ok(u64::from_le_bytes(self.read_array::<8>()?))
    }

    fn read_i16(&mut self) -> Result<i16, SaveError> {
        Ok(i16::from_le_bytes(self.read_array::<2>()?))
    }

    fn read_i32(&mut self) -> Result<i32, SaveError> {
        Ok(i32::from_le_bytes(self.read_array::<4>()?))
    }

    fn read_array<const N: usize>(&mut self) -> Result<[u8; N], SaveError> {
        let end = self
            .position
            .checked_add(N)
            .ok_or(SaveError::TruncatedBinary)?;
        let slice = self
            .bytes
            .get(self.position..end)
            .ok_or(SaveError::TruncatedBinary)?;
        let mut output = [0_u8; N];
        output.copy_from_slice(slice);
        self.position = end;
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use simulation_model::{
        BiomeClass, EntityRegistry, MapPoint, RegionId, RegionPoliticalState, RegionState,
        RegionSurface, ReliefClass, TerrainSample, TerrainState, WorldBounds, WorldSpatialState,
        WorldState, TERRAIN_SAMPLE_COUNT, TRIAL_REGION_COUNT,
    };

    use super::{decode_snapshot, encode_snapshot};
    use crate::EngineSnapshot;

    fn trial_spatial() -> WorldSpatialState {
        let regions = (1..=TRIAL_REGION_COUNT)
            .map(|index| {
                let id = RegionId(u64::try_from(index).expect("id fits u64"));
                let x = i32::try_from(index).expect("index fits i32") * 100;
                let mut neighbors = Vec::new();
                if index > 1 {
                    neighbors.push(RegionId(u64::try_from(index - 1).expect("id fits u64")));
                }
                if index < TRIAL_REGION_COUNT {
                    neighbors.push(RegionId(u64::try_from(index + 1).expect("id fits u64")));
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
        .expect("valid trial spatial state")
    }

    fn trial_terrain() -> TerrainState {
        TerrainState::new_trial(vec![
            TerrainSample {
                elevation_m: -500,
                moisture_permille: 500,
                relief: ReliefClass::ShallowOcean,
                biome: BiomeClass::Ocean,
            };
            TERRAIN_SAMPLE_COUNT
        ])
        .expect("valid trial terrain")
    }

    fn empty_snapshot(world: WorldState) -> EngineSnapshot {
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
    fn empty_snapshot_binary_round_trip_is_exact() {
        let snapshot = empty_snapshot(WorldState::new(77));
        let encoded = encode_snapshot(&snapshot).expect("snapshot should encode");
        let decoded = decode_snapshot(&encoded).expect("snapshot should decode");
        assert_eq!(decoded, snapshot);
    }

    #[test]
    fn initialized_terrain_round_trip_is_exact() {
        let mut world = WorldState::new(77);
        world.terrain = Some(trial_terrain());
        let snapshot = empty_snapshot(world);
        let encoded = encode_snapshot(&snapshot).expect("snapshot should encode");
        let decoded = decode_snapshot(&encoded).expect("snapshot should decode");
        assert_eq!(decoded, snapshot);
        assert_eq!(
            decoded
                .world
                .terrain
                .as_ref()
                .expect("terrain restored")
                .samples
                .len(),
            TERRAIN_SAMPLE_COUNT
        );
    }

    #[test]
    fn initialized_sixty_region_world_round_trip_is_exact() {
        let mut world = WorldState::new(77);
        world.spatial = trial_spatial();
        let snapshot = empty_snapshot(world);
        let encoded = encode_snapshot(&snapshot).expect("snapshot should encode");
        let decoded = decode_snapshot(&encoded).expect("snapshot should decode");
        assert_eq!(decoded, snapshot);
        assert_eq!(decoded.world.spatial.regions.len(), 60);
    }
}
