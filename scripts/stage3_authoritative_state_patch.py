from pathlib import Path
import sys

root = Path(sys.argv[1]) if len(sys.argv) > 1 else Path('.')

def rep(path, old, new):
    p = root / path
    s = p.read_text()
    if old not in s:
        raise SystemExit(f'missing anchor in {path}: {old[:90]!r}')
    p.write_text(s.replace(old, new, 1))

entity='crates/simulation-model/src/entity.rs'
rep(entity, '''impl HumanGroupState {
    #[must_use]
    pub fn is_valid(&self) -> bool {
''', '''impl HumanGroupState {
    #[must_use]
    pub fn from_initial(id: HumanGroupId, initial: &HumanGroupInitialState, seed: HumanGroupRuntimeSeed) -> Self {
        Self {
            id,
            region_id: initial.region_id,
            x_m: initial.x_m,
            z_m: initial.z_m,
            terrain_sample_index: initial.terrain_sample_index,
            population: initial.population,
            mobility_permille: initial.behavior.mobility_permille,
            food_stock_person_days: initial.food_stock_person_days,
            basic_resource_stock_units: initial.basic_resource_stock_units,
            nutrition_permille: seed.nutrition_permille,
            cohesion_permille: seed.cohesion_permille,
            risk_permille: seed.risk_permille,
            knowledge_ref: None,
            memory_ref: None,
            lineage: HumanGroupLineage::default(),
        }
    }

    #[must_use]
    pub fn is_valid(&self) -> bool {
''')
rep(entity, '''}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EntityKind {''', '''}

/// Explicit scenario input for initializing Stage 3 runtime indicators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HumanGroupRuntimeSeed {
    pub nutrition_permille: u16,
    pub cohesion_permille: u16,
    pub risk_permille: u16,
}

impl HumanGroupRuntimeSeed {
    #[must_use]
    pub const fn is_valid(self) -> bool {
        self.nutrition_permille <= 1_000 && self.cohesion_permille <= 1_000 && self.risk_permille <= 1_000
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EntityKind {''')
rep(entity, '''pub struct HumanGroupEntity {
    pub id: HumanGroupId,
    pub initial: Option<HumanGroupInitialState>,
}''', '''pub struct HumanGroupEntity {
    pub id: HumanGroupId,
    pub initial: Option<HumanGroupInitialState>,
    pub state: Option<HumanGroupState>,
}''')
rep(entity, '''                    .is_some_and(|initial| !initial.is_valid())
            {''', '''                    .is_some_and(|initial| !initial.is_valid())
                || record.state.as_ref().is_some_and(|state| state.id != record.id || !state.is_valid())
            {''')
rep(entity, '        self.create_record(None)\n', '        self.create_record(None, None)\n')
rep(entity, '''        self.create_record(Some(initial))
    }

    fn create_record(
        &mut self,
        initial: Option<HumanGroupInitialState>,
    ) -> Result<HumanGroupId, EntityRegistryError> {
        let id = HumanGroupId(allocate_id(&mut self.next_id)?);
        let previous = self.entries.insert(id, HumanGroupEntity { id, initial });''', '''        self.create_record(Some(initial), None)
    }

    /// Creates an initialized group with authoritative Stage 3 runtime state.
    ///
    /// # Errors
    /// Returns [`EntityRegistryError`] for invalid inputs or exhausted IDs.
    pub fn create_initialized_with_runtime(&mut self, initial: HumanGroupInitialState, runtime_seed: HumanGroupRuntimeSeed) -> Result<HumanGroupId, EntityRegistryError> {
        if !initial.is_valid() || !runtime_seed.is_valid() { return Err(EntityRegistryError::InvalidReference); }
        let id = HumanGroupId(allocate_id(&mut self.next_id)?);
        let state = HumanGroupState::from_initial(id, &initial, runtime_seed);
        let previous = self.entries.insert(id, HumanGroupEntity { id, initial: Some(initial), state: Some(state) });
        debug_assert!(previous.is_none());
        Ok(id)
    }

    fn create_record(&mut self, initial: Option<HumanGroupInitialState>, state: Option<HumanGroupState>) -> Result<HumanGroupId, EntityRegistryError> {
        let id = HumanGroupId(allocate_id(&mut self.next_id)?);
        let previous = self.entries.insert(id, HumanGroupEntity { id, initial, state });''')

rep('crates/simulation-model/src/lib.rs', '    HumanGroupId, HumanGroupInitialState, HumanGroupLineage, HumanGroupRegistry, HumanGroupState,\n', '    HumanGroupId, HumanGroupInitialState, HumanGroupLineage, HumanGroupRegistry, HumanGroupRuntimeSeed,\n    HumanGroupState,\n')

hg='crates/simulation-worldgen/src/human_groups.rs'
rep(hg, '    HumanGroupRegistry, InitialKnowledgeProfile, RegionId, RegionSurface, ResourceFieldState,\n', '    HumanGroupRegistry, HumanGroupRuntimeSeed, InitialKnowledgeProfile, RegionId, RegionSurface,\n    ResourceFieldState,\n')
rep(hg, '    pub settlement_bias_permille: PermilleRange,\n    pub contact_radius_m: u32,\n', '    pub settlement_bias_permille: PermilleRange,\n    pub runtime_seed: HumanGroupRuntimeSeed,\n    pub contact_radius_m: u32,\n')
rep(hg, '            || !self.settlement_bias_permille.is_valid()\n            || self.contact_radius_m == 0\n', '            || !self.settlement_bias_permille.is_valid()\n            || !self.runtime_seed.is_valid()\n            || self.contact_radius_m == 0\n')
rep(hg, '            .create_initialized(initial)\n', '            .create_initialized_with_runtime(initial, config.runtime_seed)\n')

boot='crates/simulation-worldgen/src/bootstrap.rs'
rep(boot, '    derive_terrain_effects, InitialKnowledgeProfile, MapPoint, RegionId, RegionPoliticalState,\n', '    derive_terrain_effects, HumanGroupRuntimeSeed, InitialKnowledgeProfile, MapPoint, RegionId,\n    RegionPoliticalState,\n')
rep(boot, '        settlement_bias_permille: PermilleRange { min: 200, max: 700 },\n        contact_radius_m: 400_000,\n', '        settlement_bias_permille: PermilleRange { min: 200, max: 700 },\n        runtime_seed: HumanGroupRuntimeSeed { nutrition_permille: 1_000, cohesion_permille: 500, risk_permille: 500 },\n        contact_radius_m: 400_000,\n')

save='crates/simulation-save/src/lib.rs'
rep(save, '/// Stage 2.6 adds authoritative Year-1 `HumanGroup` ecological seed state.\npub const SAVE_FORMAT_VERSION: u32 = 6;', '/// Stage 3.1 adds authoritative `HumanGroupState` persistence.\npub const SAVE_FORMAT_VERSION: u32 = 7;')
rep(save, '''        }
    }
    for city in world.entities.cities.iter() {''', '''        }
        if let Some(state) = &group.state {
            if state.id != group.id || !state.is_valid() || world.spatial.region(state.region_id).is_none() {
                return Err(SaveError::SnapshotInvariant("invalid HumanGroup runtime state"));
            }
        }
    }
    for city in world.entities.cities.iter() {''')

binary='crates/simulation-save/src/binary.rs'
rep(binary, '    HumanGroupInitialState, HumanGroupRegistry, InitialKnowledgeProfile, KnowledgeSeedValue,\n', '    HumanGroupInitialState, HumanGroupLineage, HumanGroupRegistry, HumanGroupState,\n    InitialKnowledgeProfile, KnowledgeSeedValue, KnowledgeStateRef, MemoryStateRef,\n')
rep(binary, '''    }
    Ok(())
}

fn read_human_group(cursor: &mut Cursor<'_>) -> Result<HumanGroupEntity, SaveError> {''', '''    }
    match &group.state {
        None => write_u8(bytes, 0),
        Some(state) => {
            write_u8(bytes, 1); write_u64(bytes, state.id.0); write_u64(bytes, state.region_id.0);
            write_i32(bytes, state.x_m); write_i32(bytes, state.z_m); write_u32(bytes, state.terrain_sample_index);
            write_u64(bytes, state.population); write_u16(bytes, state.mobility_permille);
            write_u64(bytes, state.food_stock_person_days); write_u64(bytes, state.basic_resource_stock_units);
            write_u16(bytes, state.nutrition_permille); write_u16(bytes, state.cohesion_permille); write_u16(bytes, state.risk_permille);
            match state.knowledge_ref { None => write_u8(bytes, 0), Some(r) => { write_u8(bytes, 1); write_u64(bytes, r.0); } }
            match state.memory_ref { None => write_u8(bytes, 0), Some(r) => { write_u8(bytes, 1); write_u64(bytes, r.0); } }
            write_optional_human_group(bytes, state.lineage.parent); write_optional_human_group(bytes, state.lineage.split_from);
            write_count(bytes, "human_group_merged_lineage", state.lineage.merged_from.len())?;
            for source in &state.lineage.merged_from { write_u64(bytes, source.0); }
        }
    }
    Ok(())
}

fn read_human_group(cursor: &mut Cursor<'_>) -> Result<HumanGroupEntity, SaveError> {''')
rep(binary, '''    };
    Ok(HumanGroupEntity { id, initial })
}''', '''    };
    let state = match cursor.read_u8()? {
        0 => None,
        1 => {
            let state_id = HumanGroupId(cursor.read_u64()?); let region_id = RegionId(cursor.read_u64()?);
            let x_m = cursor.read_i32()?; let z_m = cursor.read_i32()?; let terrain_sample_index = cursor.read_u32()?;
            let population = cursor.read_u64()?; let mobility_permille = cursor.read_u16()?;
            let food_stock_person_days = cursor.read_u64()?; let basic_resource_stock_units = cursor.read_u64()?;
            let nutrition_permille = cursor.read_u16()?; let cohesion_permille = cursor.read_u16()?; let risk_permille = cursor.read_u16()?;
            let knowledge_ref = match cursor.read_u8()? { 0 => None, 1 => Some(KnowledgeStateRef(cursor.read_u64()?)), tag => return Err(SaveError::InvalidTag { field: "HumanGroup knowledge ref", tag }) };
            let memory_ref = match cursor.read_u8()? { 0 => None, 1 => Some(MemoryStateRef(cursor.read_u64()?)), tag => return Err(SaveError::InvalidTag { field: "HumanGroup memory ref", tag }) };
            let parent = read_optional_human_group(cursor)?; let split_from = read_optional_human_group(cursor)?;
            let count = cursor.read_count("human_group_merged_lineage")?; let mut merged_from = Vec::with_capacity(count);
            for _ in 0..count { merged_from.push(HumanGroupId(cursor.read_u64()?)); }
            Some(HumanGroupState { id: state_id, region_id, x_m, z_m, terrain_sample_index, population, mobility_permille, food_stock_person_days, basic_resource_stock_units, nutrition_permille, cohesion_permille, risk_permille, knowledge_ref, memory_ref, lineage: HumanGroupLineage { parent, split_from, merged_from } })
        }
        tag => return Err(SaveError::InvalidTag { field: "HumanGroup runtime state", tag }),
    };
    Ok(HumanGroupEntity { id, initial, state })
}''')
rep(binary, 'fn write_terrain(bytes: &mut Vec<u8>, terrain: &TerrainState) -> Result<(), SaveError> {', '''fn write_optional_human_group(bytes: &mut Vec<u8>, id: Option<HumanGroupId>) {
    match id { None => write_u8(bytes, 0), Some(id) => { write_u8(bytes, 1); write_u64(bytes, id.0); } }
}
fn read_optional_human_group(cursor: &mut Cursor<'_>) -> Result<Option<HumanGroupId>, SaveError> {
    match cursor.read_u8()? { 0 => Ok(None), 1 => Ok(Some(HumanGroupId(cursor.read_u64()?))), tag => Err(SaveError::InvalidTag { field: "optional HumanGroup id", tag }) }
}

fn write_terrain(bytes: &mut Vec<u8>, terrain: &TerrainState) -> Result<(), SaveError> {''')

protocol='crates/simulation-protocol/src/lib.rs'
rep(protocol, 'pub const RENDER_SNAPSHOT_VERSION: u32 = 6;', 'pub const RENDER_SNAPSHOT_VERSION: u32 = 7;')
rep(protocol, '''                .filter_map(|group| {
                    group
                        .initial
                        .as_ref()
                        .map(|initial| RenderHumanGroupSnapshot {
                            id: group.id.0.to_string(),
                            region_id: initial.region_id.0.to_string(),
                            x_m: initial.x_m,
                            z_m: initial.z_m,
                            population: initial.population.to_string(),
                            food_stock_person_days: initial.food_stock_person_days.to_string(),
                            basic_resource_stock_units: initial
                                .basic_resource_stock_units
                                .to_string(),
                            mobility_permille: initial.behavior.mobility_permille,
                            exploration_permille: initial.behavior.exploration_permille,
                            settlement_bias_permille: initial.behavior.settlement_bias_permille,
                        })
                })''', '''                .filter_map(|group| {
                    group.state.as_ref().map(|state| RenderHumanGroupSnapshot {
                        id: group.id.0.to_string(),
                        region_id: state.region_id.0.to_string(),
                        x_m: state.x_m,
                        z_m: state.z_m,
                        population: state.population.to_string(),
                        food_stock_person_days: state.food_stock_person_days.to_string(),
                        basic_resource_stock_units: state.basic_resource_stock_units.to_string(),
                        mobility_permille: state.mobility_permille,
                        nutrition_permille: state.nutrition_permille,
                        cohesion_permille: state.cohesion_permille,
                        risk_permille: state.risk_permille,
                    })
                })''')
rep(protocol, '    pub mobility_permille: u16,\n    pub exploration_permille: u16,\n    pub settlement_bias_permille: u16,\n', '    pub mobility_permille: u16,\n    pub nutrition_permille: u16,\n    pub cohesion_permille: u16,\n    pub risk_permille: u16,\n')

rep('viewer/src/types.ts', '  mobilityPermille: number;\n  explorationPermille: number;\n  settlementBiasPermille: number;\n', '  mobilityPermille: number;\n  nutritionPermille: number;\n  cohesionPermille: number;\n  riskPermille: number;\n')
rep('viewer/src/human-group-layer.ts', '''    const settlementBias = clamp01(snapshot.settlementBiasPermille / 1000);
    const mobility = clamp01(snapshot.mobilityPermille / 1000);
    bodyMaterial.color.setHSL(0.11 - mobility * 0.025, 0.42 + settlementBias * 0.18, 0.56);
    haloMaterial.opacity = 0.25 + snapshot.explorationPermille / 1000 * 0.3;''', '''    const nutrition = clamp01(snapshot.nutritionPermille / 1000);
    const cohesion = clamp01(snapshot.cohesionPermille / 1000);
    const risk = clamp01(snapshot.riskPermille / 1000);
    const mobility = clamp01(snapshot.mobilityPermille / 1000);
    bodyMaterial.color.setHSL(0.08 + nutrition * 0.05 - risk * 0.035, 0.38 + cohesion * 0.2, 0.48 + nutrition * 0.1);
    haloMaterial.opacity = 0.2 + mobility * 0.18 + cohesion * 0.12;''')
