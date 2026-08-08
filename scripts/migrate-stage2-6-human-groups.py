from pathlib import Path


def replace_once(path: str, old: str, new: str, marker: str) -> None:
    file = Path(path)
    text = file.read_text()
    if marker in text:
        return
    if old not in text:
        raise SystemExit(f"Stage 2.6 migration anchor missing in {path}:\n{old[:180]}")
    file.write_text(text.replace(old, new, 1))


# simulation-model: add the minimal authoritative Year-1 HumanGroup seed state.
replace_once(
    "crates/simulation-model/src/entity.rs",
    """typed_entity_id!(CityId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EntityKind {""",
    """typed_entity_id!(CityId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KnowledgeSeedValue {
    pub domain_key: u32,
    pub level_permille: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct InitialKnowledgeProfile {
    pub entries: Vec<KnowledgeSeedValue>,
}

impl InitialKnowledgeProfile {
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.entries.iter().all(|entry| entry.level_permille <= 1_000)
            && self
                .entries
                .windows(2)
                .all(|pair| pair[0].domain_key < pair[1].domain_key)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HumanGroupBehaviorProfile {
    pub mobility_permille: u16,
    pub exploration_permille: u16,
    pub settlement_bias_permille: u16,
}

impl HumanGroupBehaviorProfile {
    #[must_use]
    pub const fn is_valid(self) -> bool {
        self.mobility_permille <= 1_000
            && self.exploration_permille <= 1_000
            && self.settlement_bias_permille <= 1_000
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HumanGroupInitialState {
    pub region_id: RegionId,
    pub x_m: i32,
    pub z_m: i32,
    pub terrain_sample_index: u32,
    pub population: u64,
    pub food_stock_person_days: u64,
    pub basic_resource_stock_units: u64,
    pub behavior: HumanGroupBehaviorProfile,
    pub knowledge: InitialKnowledgeProfile,
}

impl HumanGroupInitialState {
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.region_id.0 != 0
            && self.population != 0
            && self.behavior.is_valid()
            && self.knowledge.is_valid()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EntityKind {""",
    "pub struct HumanGroupInitialState",
)

replace_once(
    "crates/simulation-model/src/entity.rs",
    """identity_registry!(HumanGroupId, HumanGroupEntity, HumanGroupRegistry);
identity_registry!(SettlementId, SettlementEntity, SettlementRegistry);""",
    """#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HumanGroupEntity {
    pub id: HumanGroupId,
    pub initial: Option<HumanGroupInitialState>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HumanGroupRegistry {
    entries: BTreeMap<HumanGroupId, HumanGroupEntity>,
    next_id: u64,
}

impl HumanGroupRegistry {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
            next_id: StableEntityId::FIRST.0,
        }
    }

    /// Restores a HumanGroup registry from canonical persisted parts.
    ///
    /// # Errors
    /// Returns [`EntityRegistryError`] for invalid identity, allocator or seed-state data.
    pub fn from_parts(
        next_id: u64,
        records: Vec<HumanGroupEntity>,
    ) -> Result<Self, EntityRegistryError> {
        let mut entries = BTreeMap::new();
        for record in records {
            if record.id.0 == 0
                || record
                    .initial
                    .as_ref()
                    .is_some_and(|initial| !initial.is_valid())
            {
                return Err(EntityRegistryError::InvalidReference);
            }
            if entries.insert(record.id, record).is_some() {
                return Err(EntityRegistryError::DuplicateId);
            }
        }
        validate_next_id(next_id, entries.keys().map(|id| id.0))?;
        Ok(Self { entries, next_id })
    }

    /// Creates an uninitialized HumanGroup identity for generic lifecycle tests
    /// and later runtime systems.
    ///
    /// # Errors
    /// Returns [`EntityRegistryError::IdExhausted`] if ID space is exhausted.
    pub fn create(&mut self) -> Result<HumanGroupId, EntityRegistryError> {
        self.create_record(None)
    }

    /// Creates a Year-1 HumanGroup with its authoritative initial ecological state.
    ///
    /// # Errors
    /// Returns [`EntityRegistryError`] if the seed state is invalid or ID space is exhausted.
    pub fn create_initialized(
        &mut self,
        initial: HumanGroupInitialState,
    ) -> Result<HumanGroupId, EntityRegistryError> {
        if !initial.is_valid() {
            return Err(EntityRegistryError::InvalidReference);
        }
        self.create_record(Some(initial))
    }

    fn create_record(
        &mut self,
        initial: Option<HumanGroupInitialState>,
    ) -> Result<HumanGroupId, EntityRegistryError> {
        let id = HumanGroupId(allocate_id(&mut self.next_id)?);
        let previous = self.entries.insert(id, HumanGroupEntity { id, initial });
        debug_assert!(previous.is_none());
        Ok(id)
    }

    /// Removes an existing HumanGroup without recycling its stable ID.
    ///
    /// # Errors
    /// Returns [`EntityRegistryError::UnknownEntity`] if absent.
    pub fn remove(
        &mut self,
        id: HumanGroupId,
    ) -> Result<HumanGroupEntity, EntityRegistryError> {
        self.entries
            .remove(&id)
            .ok_or(EntityRegistryError::UnknownEntity)
    }

    #[must_use]
    pub const fn next_id(&self) -> u64 {
        self.next_id
    }
}

impl Default for HumanGroupRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl EntityRegistry for HumanGroupRegistry {
    type Id = HumanGroupId;
    type Record = HumanGroupEntity;
    type Iter<'a> = std::collections::btree_map::Values<'a, HumanGroupId, HumanGroupEntity>;

    fn len(&self) -> usize {
        self.entries.len()
    }

    fn get(&self, id: Self::Id) -> Option<&Self::Record> {
        self.entries.get(&id)
    }

    fn iter(&self) -> Self::Iter<'_> {
        self.entries.values()
    }
}

identity_registry!(SettlementId, SettlementEntity, SettlementRegistry);""",
    "pub fn create_initialized(",
)

replace_once(
    "crates/simulation-model/src/lib.rs",
    """    EntityRegistryError, EntityWorldState, HumanGroupEntity, HumanGroupId, HumanGroupRegistry,
    PoliticalEntity, PoliticalEntityId, PoliticalEntityRegistry, RegionId, SettlementEntity,""",
    """    EntityRegistryError, EntityWorldState, HumanGroupBehaviorProfile, HumanGroupEntity,
    HumanGroupId, HumanGroupInitialState, HumanGroupRegistry, InitialKnowledgeProfile,
    KnowledgeSeedValue, PoliticalEntity, PoliticalEntityId, PoliticalEntityRegistry, RegionId,
    SettlementEntity,""",
    "HumanGroupInitialState",
)

# worldgen exports
replace_once(
    "crates/simulation-worldgen/src/lib.rs",
    """mod hydrology;
mod resources;

use hydrology::ensure_seeded_island;""",
    """mod human_groups;
mod hydrology;
mod resources;

pub use human_groups::{
    generate_initial_human_groups, initial_group_contacts, initial_human_group_report,
    HumanGroupGenerationError, InitialGroupContact, InitialHumanGroupConfig,
    InitialHumanGroupReport, PermilleRange,
};
use hydrology::ensure_seeded_island;""",
    "mod human_groups;",
)

# Save v6: HumanGroup initial state becomes authoritative persistence.
replace_once(
    "crates/simulation-save/src/lib.rs",
    """/// Stage 2.5 adds dynamic entity registries and stable allocator state.
pub const SAVE_FORMAT_VERSION: u32 = 5;""",
    """/// Stage 2.6 adds authoritative Year-1 HumanGroup ecological seed state.
pub const SAVE_FORMAT_VERSION: u32 = 6;""",
    "SAVE_FORMAT_VERSION: u32 = 6",
)

replace_once(
    "crates/simulation-save/src/lib.rs",
    """    for city in world.entities.cities.iter() {
        if city""",
    """    for group in world.entities.human_groups.iter() {
        if let Some(initial) = &group.initial {
            if !initial.is_valid()
                || world.spatial.region(initial.region_id).is_none()
                || world.spatial.bounds.is_some_and(|bounds| {
                    initial.x_m < bounds.min_x_m
                        || initial.x_m > bounds.max_x_m
                        || initial.z_m < bounds.min_z_m
                        || initial.z_m > bounds.max_z_m
                })
                || world.terrain.as_ref().is_some_and(|terrain| {
                    usize::try_from(initial.terrain_sample_index)
                        .map_or(true, |index| index >= terrain.samples.len())
                })
            {
                return Err(SaveError::SnapshotInvariant(
                    \"HumanGroup initial state references invalid geography\",
                ));
            }
        }
    }
    for city in world.entities.cities.iter() {
        if city""",
    "HumanGroup initial state references invalid geography",
)

# Binary import additions.
replace_once(
    "crates/simulation-save/src/binary.rs",
    """    EventRecord, EventSource, HumanGroupEntity, HumanGroupId, HumanGroupRegistry, MapPoint,
    PoliticalEntity, PoliticalEntityId, PoliticalEntityRegistry, QueuedCommand, RegionDraft,""",
    """    EventRecord, EventSource, HumanGroupBehaviorProfile, HumanGroupEntity, HumanGroupId,
    HumanGroupInitialState, HumanGroupRegistry, InitialKnowledgeProfile, KnowledgeSeedValue,
    MapPoint, PoliticalEntity, PoliticalEntityId, PoliticalEntityRegistry, QueuedCommand, RegionDraft,""",
    "HumanGroupInitialState",
)

replace_once(
    "crates/simulation-save/src/binary.rs",
    """    for group in world.entities.human_groups.iter() {
        write_u64(bytes, group.id.0);
    }""",
    """    for group in world.entities.human_groups.iter() {
        write_human_group(bytes, group)?;
    }""",
    "write_human_group(bytes, group)?",
)

replace_once(
    "crates/simulation-save/src/binary.rs",
    """    for _ in 0..human_group_count {
        human_groups.push(HumanGroupEntity {
            id: HumanGroupId(cursor.read_u64()?),
        });
    }""",
    """    for _ in 0..human_group_count {
        human_groups.push(read_human_group(cursor)?);
    }""",
    "read_human_group(cursor)?",
)

replace_once(
    "crates/simulation-save/src/binary.rs",
    """fn write_terrain(bytes: &mut Vec<u8>, terrain: &TerrainState) -> Result<(), SaveError> {""",
    """fn write_human_group(
    bytes: &mut Vec<u8>,
    group: &HumanGroupEntity,
) -> Result<(), SaveError> {
    write_u64(bytes, group.id.0);
    match &group.initial {
        None => write_u8(bytes, 0),
        Some(initial) => {
            write_u8(bytes, 1);
            write_u64(bytes, initial.region_id.0);
            write_i32(bytes, initial.x_m);
            write_i32(bytes, initial.z_m);
            write_u32(bytes, initial.terrain_sample_index);
            write_u64(bytes, initial.population);
            write_u64(bytes, initial.food_stock_person_days);
            write_u64(bytes, initial.basic_resource_stock_units);
            write_u16(bytes, initial.behavior.mobility_permille);
            write_u16(bytes, initial.behavior.exploration_permille);
            write_u16(bytes, initial.behavior.settlement_bias_permille);
            write_count(bytes, \"knowledge_seed_values\", initial.knowledge.entries.len())?;
            for entry in &initial.knowledge.entries {
                write_u32(bytes, entry.domain_key);
                write_u16(bytes, entry.level_permille);
            }
        }
    }
    Ok(())
}

fn read_human_group(cursor: &mut Cursor<'_>) -> Result<HumanGroupEntity, SaveError> {
    let id = HumanGroupId(cursor.read_u64()?);
    let initial = match cursor.read_u8()? {
        0 => None,
        1 => {
            let region_id = RegionId(cursor.read_u64()?);
            let x_m = cursor.read_i32()?;
            let z_m = cursor.read_i32()?;
            let terrain_sample_index = cursor.read_u32()?;
            let population = cursor.read_u64()?;
            let food_stock_person_days = cursor.read_u64()?;
            let basic_resource_stock_units = cursor.read_u64()?;
            let behavior = HumanGroupBehaviorProfile {
                mobility_permille: cursor.read_u16()?,
                exploration_permille: cursor.read_u16()?,
                settlement_bias_permille: cursor.read_u16()?,
            };
            let knowledge_count = cursor.read_count(\"knowledge_seed_values\")?;
            let mut entries = Vec::with_capacity(knowledge_count);
            for _ in 0..knowledge_count {
                entries.push(KnowledgeSeedValue {
                    domain_key: cursor.read_u32()?,
                    level_permille: cursor.read_u16()?,
                });
            }
            Some(HumanGroupInitialState {
                region_id,
                x_m,
                z_m,
                terrain_sample_index,
                population,
                food_stock_person_days,
                basic_resource_stock_units,
                behavior,
                knowledge: InitialKnowledgeProfile { entries },
            })
        }
        tag => {
            return Err(SaveError::InvalidTag {
                field: \"HumanGroup initial state\",
                tag,
            });
        }
    };
    Ok(HumanGroupEntity { id, initial })
}

fn write_terrain(bytes: &mut Vec<u8>, terrain: &TerrainState) -> Result<(), SaveError> {""",
    "fn write_human_group(",
)

# RenderSnapshot v5 exposes initialized HumanGroups without rendering them yet.
replace_once(
    "crates/simulation-protocol/src/lib.rs",
    """use simulation_model::{
    BiomeClass, MapPoint, RegionSurface, ReliefClass, TerrainState, WorldBounds, WorldState,
};

pub const RENDER_SNAPSHOT_VERSION: u32 = 4;""",
    """use simulation_model::{
    BiomeClass, EntityRegistry, MapPoint, RegionSurface, ReliefClass, TerrainState, WorldBounds,
    WorldState,
};

pub const RENDER_SNAPSHOT_VERSION: u32 = 5;""",
    "RENDER_SNAPSHOT_VERSION: u32 = 5",
)

replace_once(
    "crates/simulation-protocol/src/lib.rs",
    """    pub terrain: Option<RenderTerrainSnapshot>,
    pub regions: Vec<RenderRegionSnapshot>,
}""",
    """    pub terrain: Option<RenderTerrainSnapshot>,
    pub regions: Vec<RenderRegionSnapshot>,
    pub human_groups: Vec<RenderHumanGroupSnapshot>,
}""",
    "pub human_groups: Vec<RenderHumanGroupSnapshot>",
)

replace_once(
    "crates/simulation-protocol/src/lib.rs",
    """            regions: world
                .spatial
                .regions
                .iter()
                .map(RenderRegionSnapshot::from)
                .collect(),
        }
    }
}""",
    """            regions: world
                .spatial
                .regions
                .iter()
                .map(RenderRegionSnapshot::from)
                .collect(),
            human_groups: world
                .entities
                .human_groups
                .iter()
                .filter_map(|group| {
                    group.initial.as_ref().map(|initial| RenderHumanGroupSnapshot {
                        id: group.id.0.to_string(),
                        region_id: initial.region_id.0.to_string(),
                        x_m: initial.x_m,
                        z_m: initial.z_m,
                        population: initial.population,
                        food_stock_person_days: initial.food_stock_person_days,
                        basic_resource_stock_units: initial.basic_resource_stock_units,
                        mobility_permille: initial.behavior.mobility_permille,
                        exploration_permille: initial.behavior.exploration_permille,
                        settlement_bias_permille: initial.behavior.settlement_bias_permille,
                    })
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = \"camelCase\")]
pub struct RenderHumanGroupSnapshot {
    pub id: String,
    pub region_id: String,
    pub x_m: i32,
    pub z_m: i32,
    pub population: u64,
    pub food_stock_person_days: u64,
    pub basic_resource_stock_units: u64,
    pub mobility_permille: u16,
    pub exploration_permille: u16,
    pub settlement_bias_permille: u16,
}""",
    "pub struct RenderHumanGroupSnapshot",
)

# TypeScript protocol mirror.
replace_once(
    "viewer/src/types.ts",
    """export interface RenderWorldSnapshot {
  initialized: boolean;
  bounds: RenderWorldBounds | null;
  terrain: RenderTerrainSnapshot | null;
  regions: RenderRegionSnapshot[];
}""",
    """export interface RenderHumanGroupSnapshot {
  id: string;
  regionId: string;
  xM: number;
  zM: number;
  population: number;
  foodStockPersonDays: number;
  basicResourceStockUnits: number;
  mobilityPermille: number;
  explorationPermille: number;
  settlementBiasPermille: number;
}

export interface RenderWorldSnapshot {
  initialized: boolean;
  bounds: RenderWorldBounds | null;
  terrain: RenderTerrainSnapshot | null;
  regions: RenderRegionSnapshot[];
  humanGroups: RenderHumanGroupSnapshot[];
}""",
    "humanGroups: RenderHumanGroupSnapshot[]",
)
