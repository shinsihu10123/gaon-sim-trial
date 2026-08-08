use std::collections::BTreeMap;

/// Non-zero, monotonic identity token used as the common storage primitive for
/// dynamic simulation entities.
///
/// IDs are never recycled after removal. Deterministic registries issue IDs in
/// ascending order, so identical initial state and mutation order produce the
/// same identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StableEntityId(pub u64);

impl StableEntityId {
    pub const FIRST: Self = Self(1);

    #[must_use]
    pub const fn is_valid(self) -> bool {
        self.0 != 0
    }
}

macro_rules! typed_entity_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(pub u64);

        impl $name {
            #[must_use]
            pub const fn stable(self) -> StableEntityId {
                StableEntityId(self.0)
            }
        }

        impl From<$name> for StableEntityId {
            fn from(id: $name) -> Self {
                id.stable()
            }
        }
    };
}

typed_entity_id!(HumanGroupId);
typed_entity_id!(SettlementId);
typed_entity_id!(CommunityId);
typed_entity_id!(PoliticalEntityId);
typed_entity_id!(CountryId);
typed_entity_id!(RegionId);
typed_entity_id!(CityId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EntityKind {
    Country,
    Region,
    City,
    HumanGroup,
    Settlement,
    Community,
    PoliticalEntity,
}

/// Type-tagged identity used by Event Ledger and generic reference diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityRef {
    pub kind: EntityKind,
    pub id: StableEntityId,
}

impl EntityRef {
    #[must_use]
    pub const fn human_group(id: HumanGroupId) -> Self {
        Self {
            kind: EntityKind::HumanGroup,
            id: id.stable(),
        }
    }

    #[must_use]
    pub const fn settlement(id: SettlementId) -> Self {
        Self {
            kind: EntityKind::Settlement,
            id: id.stable(),
        }
    }

    #[must_use]
    pub const fn community(id: CommunityId) -> Self {
        Self {
            kind: EntityKind::Community,
            id: id.stable(),
        }
    }

    #[must_use]
    pub const fn political_entity(id: PoliticalEntityId) -> Self {
        Self {
            kind: EntityKind::PoliticalEntity,
            id: id.stable(),
        }
    }

    #[must_use]
    pub const fn country(id: CountryId) -> Self {
        Self {
            kind: EntityKind::Country,
            id: id.stable(),
        }
    }

    #[must_use]
    pub const fn region(id: RegionId) -> Self {
        Self {
            kind: EntityKind::Region,
            id: id.stable(),
        }
    }

    #[must_use]
    pub const fn city(id: CityId) -> Self {
        Self {
            kind: EntityKind::City,
            id: id.stable(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityRegistryError {
    InvalidId,
    DuplicateId,
    NonMonotonicNextId,
    IdExhausted,
    UnknownEntity,
    ReferenceInUse,
    InvalidReference,
}

impl core::fmt::Display for EntityRegistryError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(match self {
            Self::InvalidId => "entity id must be non-zero",
            Self::DuplicateId => "entity registry contains a duplicate id",
            Self::NonMonotonicNextId => "registry next id must be greater than every stored id",
            Self::IdExhausted => "entity identifier space exhausted",
            Self::UnknownEntity => "entity does not exist",
            Self::ReferenceInUse => "entity is still referenced by another authoritative entity",
            Self::InvalidReference => "entity contains a reference to an unknown entity",
        })
    }
}

impl std::error::Error for EntityRegistryError {}

/// Common read-only contract shared by dynamic registries.
pub trait EntityRegistry {
    type Id: Copy + Ord;
    type Record;
    type Iter<'a>: Iterator<Item = &'a Self::Record>
    where
        Self: 'a;

    fn len(&self) -> usize;
    fn get(&self, id: Self::Id) -> Option<&Self::Record>;
    fn iter(&self) -> Self::Iter<'_>;

    #[must_use]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[must_use]
    fn contains(&self, id: Self::Id) -> bool {
        self.get(id).is_some()
    }
}

macro_rules! identity_registry {
    ($id:ident, $record:ident, $registry:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $record {
            pub id: $id,
        }

        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $registry {
            entries: BTreeMap<$id, $record>,
            next_id: u64,
        }

        impl $registry {
            #[must_use]
            pub const fn new() -> Self {
                Self {
                    entries: BTreeMap::new(),
                    next_id: StableEntityId::FIRST.0,
                }
            }

            /// Restores a registry from canonical persisted parts.
            ///
            /// # Errors
            /// Returns [`EntityRegistryError`] for invalid IDs or allocator state.
            pub fn from_parts(
                next_id: u64,
                records: Vec<$record>,
            ) -> Result<Self, EntityRegistryError> {
                let mut entries = BTreeMap::new();
                for record in records {
                    if record.id.0 == 0 {
                        return Err(EntityRegistryError::InvalidId);
                    }
                    if entries.insert(record.id, record).is_some() {
                        return Err(EntityRegistryError::DuplicateId);
                    }
                }
                validate_next_id(next_id, entries.keys().map(|id| id.0))?;
                Ok(Self { entries, next_id })
            }

            /// Creates one identity using the next deterministic stable ID.
            ///
            /// # Errors
            /// Returns [`EntityRegistryError::IdExhausted`] if ID space is exhausted.
            pub fn create(&mut self) -> Result<$id, EntityRegistryError> {
                let id = $id(allocate_id(&mut self.next_id)?);
                let previous = self.entries.insert(id, $record { id });
                debug_assert!(previous.is_none());
                Ok(id)
            }

            /// Removes one existing identity without recycling its ID.
            ///
            /// # Errors
            /// Returns [`EntityRegistryError::UnknownEntity`] if absent.
            pub fn remove(&mut self, id: $id) -> Result<$record, EntityRegistryError> {
                self.entries
                    .remove(&id)
                    .ok_or(EntityRegistryError::UnknownEntity)
            }

            #[must_use]
            pub const fn next_id(&self) -> u64 {
                self.next_id
            }
        }

        impl Default for $registry {
            fn default() -> Self {
                Self::new()
            }
        }

        impl EntityRegistry for $registry {
            type Id = $id;
            type Record = $record;
            type Iter<'a> = std::collections::btree_map::Values<'a, $id, $record>;

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
    };
}

identity_registry!(HumanGroupId, HumanGroupEntity, HumanGroupRegistry);
identity_registry!(SettlementId, SettlementEntity, SettlementRegistry);
identity_registry!(CommunityId, CommunityEntity, CommunityRegistry);
identity_registry!(PoliticalEntityId, PoliticalEntity, PoliticalEntityRegistry);
identity_registry!(CountryId, CountryEntity, CountryRegistry);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CityEntity {
    pub id: CityId,
    pub country: Option<CountryId>,
    pub region: Option<RegionId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CityRegistry {
    entries: BTreeMap<CityId, CityEntity>,
    next_id: u64,
}

impl CityRegistry {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
            next_id: StableEntityId::FIRST.0,
        }
    }

    /// Restores a city registry from canonical persisted parts.
    ///
    /// # Errors
    /// Returns [`EntityRegistryError`] for invalid IDs or allocator state.
    pub fn from_parts(next_id: u64, records: Vec<CityEntity>) -> Result<Self, EntityRegistryError> {
        let mut entries = BTreeMap::new();
        for record in records {
            if record.id.0 == 0 {
                return Err(EntityRegistryError::InvalidId);
            }
            if entries.insert(record.id, record).is_some() {
                return Err(EntityRegistryError::DuplicateId);
            }
        }
        validate_next_id(next_id, entries.keys().map(|id| id.0))?;
        Ok(Self { entries, next_id })
    }

    /// Creates a city identity with optional country and Region references.
    /// Reference existence is validated at the aggregate world layer.
    ///
    /// # Errors
    /// Returns [`EntityRegistryError::IdExhausted`] if no further ID can be
    /// issued.
    pub fn create(
        &mut self,
        country: Option<CountryId>,
        region: Option<RegionId>,
    ) -> Result<CityId, EntityRegistryError> {
        let id = CityId(allocate_id(&mut self.next_id)?);
        let previous = self.entries.insert(
            id,
            CityEntity {
                id,
                country,
                region,
            },
        );
        debug_assert!(previous.is_none());
        Ok(id)
    }

    /// Removes an existing city identity.
    ///
    /// # Errors
    /// Returns [`EntityRegistryError::UnknownEntity`] if the ID is absent.
    pub fn remove(&mut self, id: CityId) -> Result<CityEntity, EntityRegistryError> {
        self.entries
            .remove(&id)
            .ok_or(EntityRegistryError::UnknownEntity)
    }

    #[must_use]
    pub const fn next_id(&self) -> u64 {
        self.next_id
    }
}

impl Default for CityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl EntityRegistry for CityRegistry {
    type Id = CityId;
    type Record = CityEntity;
    type Iter<'a> = std::collections::btree_map::Values<'a, CityId, CityEntity>;

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

/// Non-spatial dynamic entity registries owned by `WorldState`.
///
/// Civilization-origin worlds legitimately begin with zero countries. The
/// pre-state registries are first-class authoritative state and can evolve
/// before any `Country` exists.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EntityWorldState {
    pub human_groups: HumanGroupRegistry,
    pub settlements: SettlementRegistry,
    pub communities: CommunityRegistry,
    pub political_entities: PoliticalEntityRegistry,
    pub countries: CountryRegistry,
    pub cities: CityRegistry,
}

fn allocate_id(next_id: &mut u64) -> Result<u64, EntityRegistryError> {
    if *next_id == 0 {
        return Err(EntityRegistryError::InvalidId);
    }
    let id = *next_id;
    *next_id = next_id
        .checked_add(1)
        .ok_or(EntityRegistryError::IdExhausted)?;
    Ok(id)
}

fn validate_next_id(
    next_id: u64,
    ids: impl Iterator<Item = u64>,
) -> Result<(), EntityRegistryError> {
    if next_id == 0 {
        return Err(EntityRegistryError::InvalidId);
    }
    if ids.max().is_some_and(|maximum| next_id <= maximum) {
        return Err(EntityRegistryError::NonMonotonicNextId);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        CityRegistry, CountryRegistry, EntityRegistry, EntityRegistryError, HumanGroupRegistry,
        PoliticalEntityRegistry, SettlementRegistry, StableEntityId,
    };

    #[test]
    fn country_ids_are_monotonic_and_never_reused() {
        let mut registry = CountryRegistry::new();
        let first = registry.create().expect("first id");
        let second = registry.create().expect("second id");
        registry.remove(first).expect("remove first");
        let third = registry.create().expect("third id");

        assert_eq!(first.0, StableEntityId::FIRST.0);
        assert_eq!(second.0, 2);
        assert_eq!(third.0, 3);
        assert!(!registry.contains(first));
        assert!(registry.contains(second));
        assert!(registry.contains(third));
    }

    #[test]
    fn civilization_origin_registries_share_monotonic_contract() {
        let mut groups = HumanGroupRegistry::new();
        let first_group = groups.create().expect("group 1");
        groups.remove(first_group).expect("remove group");
        assert_eq!(groups.create().expect("group 2").0, 2);

        let mut settlements = SettlementRegistry::new();
        assert_eq!(settlements.create().expect("settlement").0, 1);

        let mut political_entities = PoliticalEntityRegistry::new();
        assert_eq!(political_entities.create().expect("polity").0, 1);
    }

    #[test]
    fn registry_iteration_is_id_sorted() {
        let registry = CountryRegistry::from_parts(
            10,
            vec![
                super::CountryEntity {
                    id: super::CountryId(7),
                },
                super::CountryEntity {
                    id: super::CountryId(2),
                },
            ],
        )
        .expect("canonical allocator state");
        let ids: Vec<_> = registry.iter().map(|record| record.id.0).collect();
        assert_eq!(ids, vec![2, 7]);
    }

    #[test]
    fn restored_registry_rejects_allocator_reuse() {
        assert_eq!(
            CountryRegistry::from_parts(
                7,
                vec![super::CountryEntity {
                    id: super::CountryId(7),
                }]
            ),
            Err(EntityRegistryError::NonMonotonicNextId)
        );
    }

    #[test]
    fn city_registry_preserves_optional_references() {
        let mut registry = CityRegistry::new();
        let city = registry
            .create(Some(super::CountryId(4)), Some(super::RegionId(9)))
            .expect("city id");
        let record = registry.get(city).expect("city exists");
        assert_eq!(record.country, Some(super::CountryId(4)));
        assert_eq!(record.region, Some(super::RegionId(9)));
    }
}
