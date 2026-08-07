use std::collections::BTreeMap;

use crate::{CountryId, EntityRegistry, EntityRegistryError, RegionId, StableEntityId};

/// Region count used by the Standard Benchmark S fixture only.
pub const TRIAL_REGION_COUNT: usize = 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MapPoint {
    pub x_m: i32,
    pub z_m: i32,
}

impl MapPoint {
    #[must_use]
    pub const fn new(x_m: i32, z_m: i32) -> Self {
        Self { x_m, z_m }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorldBounds {
    pub min_x_m: i32,
    pub max_x_m: i32,
    pub min_z_m: i32,
    pub max_z_m: i32,
}

impl WorldBounds {
    /// # Errors
    /// Returns [`WorldSpatialError::InvalidBounds`] for zero/negative extent.
    pub const fn new(
        west_m: i32,
        east_m: i32,
        south_m: i32,
        north_m: i32,
    ) -> Result<Self, WorldSpatialError> {
        if west_m >= east_m || south_m >= north_m {
            return Err(WorldSpatialError::InvalidBounds);
        }
        Ok(Self {
            min_x_m: west_m,
            max_x_m: east_m,
            min_z_m: south_m,
            max_z_m: north_m,
        })
    }

    #[must_use]
    pub const fn contains(self, point: MapPoint) -> bool {
        point.x_m >= self.min_x_m
            && point.x_m <= self.max_x_m
            && point.z_m >= self.min_z_m
            && point.z_m <= self.max_z_m
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegionSurface {
    Land,
    Ocean,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct RegionPoliticalState {
    pub legal_owner: Option<CountryId>,
    pub controller: Option<CountryId>,
}

impl RegionPoliticalState {
    #[must_use]
    pub const fn unclaimed() -> Self {
        Self {
            legal_owner: None,
            controller: None,
        }
    }

    #[must_use]
    pub const fn sovereign(country_id: CountryId) -> Self {
        Self {
            legal_owner: Some(country_id),
            controller: Some(country_id),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RegionState {
    pub id: RegionId,
    pub surface: RegionSurface,
    pub center: MapPoint,
    pub boundary: Vec<MapPoint>,
    pub neighbors: Vec<RegionId>,
    pub political: RegionPoliticalState,
}

/// Region creation input before the registry assigns a stable ID.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RegionDraft {
    pub surface: RegionSurface,
    pub center: MapPoint,
    pub boundary: Vec<MapPoint>,
    pub neighbors: Vec<RegionId>,
    pub political: RegionPoliticalState,
}

/// Deterministic Region registry.
///
/// The BTreeMap key is the canonical iteration order. Removed IDs are never
/// reused because `next_id` only advances.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionRegistry {
    entries: BTreeMap<RegionId, RegionState>,
    next_id: u64,
}

impl RegionRegistry {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
            next_id: StableEntityId::FIRST.0,
        }
    }

    /// Builds a registry around already-identified world-generation records.
    ///
    /// # Errors
    /// Returns [`EntityRegistryError`] when IDs are invalid or duplicated.
    pub fn from_records(records: Vec<RegionState>) -> Result<Self, EntityRegistryError> {
        let next_id = records
            .iter()
            .map(|record| record.id.0)
            .max()
            .unwrap_or(0)
            .checked_add(1)
            .ok_or(EntityRegistryError::IdExhausted)?;
        let next_id = next_id.max(StableEntityId::FIRST.0);
        Self::from_parts(next_id, records)
    }

    /// Restores a registry from canonical persisted allocator state and records.
    ///
    /// # Errors
    /// Returns [`EntityRegistryError`] for invalid IDs, duplicates, record/key
    /// inconsistency, or a `next_id` that could reuse an existing ID.
    pub fn from_parts(
        next_id: u64,
        records: Vec<RegionState>,
    ) -> Result<Self, EntityRegistryError> {
        if next_id == 0 {
            return Err(EntityRegistryError::InvalidId);
        }
        let mut entries = BTreeMap::new();
        for record in records {
            if record.id.0 == 0 {
                return Err(EntityRegistryError::InvalidId);
            }
            if entries.insert(record.id, record).is_some() {
                return Err(EntityRegistryError::DuplicateId);
            }
        }
        if entries.keys().next_back().is_some_and(|id| next_id <= id.0) {
            return Err(EntityRegistryError::NonMonotonicNextId);
        }
        Ok(Self { entries, next_id })
    }

    /// Allocates and inserts a Region, updating reciprocal neighbor references.
    ///
    /// # Errors
    /// Returns [`EntityRegistryError`] for unknown neighbors or exhausted IDs.
    pub fn create(&mut self, draft: RegionDraft) -> Result<RegionId, EntityRegistryError> {
        if draft.neighbors.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(EntityRegistryError::InvalidReference);
        }
        if draft
            .neighbors
            .iter()
            .any(|neighbor| !self.entries.contains_key(neighbor))
        {
            return Err(EntityRegistryError::InvalidReference);
        }

        let id = RegionId(self.next_id);
        self.next_id = self
            .next_id
            .checked_add(1)
            .ok_or(EntityRegistryError::IdExhausted)?;

        let neighbors = draft.neighbors.clone();
        self.entries.insert(
            id,
            RegionState {
                id,
                surface: draft.surface,
                center: draft.center,
                boundary: draft.boundary,
                neighbors: draft.neighbors,
                political: draft.political,
            },
        );

        for neighbor in neighbors {
            let counterpart = self
                .entries
                .get_mut(&neighbor)
                .ok_or(EntityRegistryError::InvalidReference)?;
            match counterpart.neighbors.binary_search(&id) {
                Ok(_) => {}
                Err(position) => counterpart.neighbors.insert(position, id),
            }
        }

        Ok(id)
    }

    /// Removes a Region and deletes reciprocal adjacency references.
    ///
    /// # Errors
    /// Returns [`EntityRegistryError::UnknownEntity`] if absent.
    pub fn remove(&mut self, id: RegionId) -> Result<RegionState, EntityRegistryError> {
        let removed = self
            .entries
            .remove(&id)
            .ok_or(EntityRegistryError::UnknownEntity)?;
        for neighbor in &removed.neighbors {
            if let Some(counterpart) = self.entries.get_mut(neighbor) {
                if let Ok(position) = counterpart.neighbors.binary_search(&id) {
                    counterpart.neighbors.remove(position);
                }
            }
        }
        Ok(removed)
    }

    #[must_use]
    pub fn get_mut(&mut self, id: RegionId) -> Option<&mut RegionState> {
        self.entries.get_mut(&id)
    }

    #[must_use]
    pub const fn next_id(&self) -> u64 {
        self.next_id
    }
}

impl Default for RegionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl EntityRegistry for RegionRegistry {
    type Id = RegionId;
    type Record = RegionState;
    type Iter<'a> = std::collections::btree_map::Values<'a, RegionId, RegionState>;

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

impl<'a> IntoIterator for &'a RegionRegistry {
    type Item = &'a RegionState;
    type IntoIter = std::collections::btree_map::Values<'a, RegionId, RegionState>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.values()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldSpatialState {
    pub bounds: Option<WorldBounds>,
    pub regions: RegionRegistry,
}

impl WorldSpatialState {
    #[must_use]
    pub const fn uninitialized() -> Self {
        Self {
            bounds: None,
            regions: RegionRegistry::new(),
        }
    }

    /// # Errors
    /// Returns [`WorldSpatialError`] for invalid topology.
    pub fn new(bounds: WorldBounds, regions: Vec<RegionState>) -> Result<Self, WorldSpatialError> {
        let registry = RegionRegistry::from_records(regions)
            .map_err(|_| WorldSpatialError::InvalidRegionRegistry)?;
        Self::from_registry(bounds, registry)
    }

    /// # Errors
    /// Returns the same validation errors as [`Self::new`].
    pub fn new_trial(
        bounds: WorldBounds,
        regions: Vec<RegionState>,
    ) -> Result<Self, WorldSpatialError> {
        Self::new(bounds, regions)
    }

    /// Restores a spatial state with persisted Region allocator state.
    ///
    /// # Errors
    /// Returns [`WorldSpatialError`] for invalid topology.
    pub fn from_registry(
        bounds: WorldBounds,
        regions: RegionRegistry,
    ) -> Result<Self, WorldSpatialError> {
        let spatial = Self {
            bounds: Some(bounds),
            regions,
        };
        spatial.validate()?;
        Ok(spatial)
    }

    #[must_use]
    pub fn is_initialized(&self) -> bool {
        self.bounds.is_some()
    }

    #[must_use]
    pub fn region(&self, id: RegionId) -> Option<&RegionState> {
        self.regions.get(id)
    }

    /// Creates a Region in an initialized world.
    ///
    /// # Errors
    /// Returns [`WorldSpatialError`] for invalid geometry, ownership, references
    /// or registry allocation failure.
    pub fn create_region(&mut self, draft: RegionDraft) -> Result<RegionId, WorldSpatialError> {
        let bounds = self
            .bounds
            .ok_or(WorldSpatialError::PartialInitialization)?;
        validate_region_draft(bounds, &draft)?;
        let id = self
            .regions
            .create(draft)
            .map_err(|_| WorldSpatialError::InvalidRegionRegistry)?;
        self.validate()?;
        Ok(id)
    }

    /// Removes a Region and reciprocal adjacency references.
    ///
    /// # Errors
    /// Returns [`WorldSpatialError`] if the Region does not exist or the
    /// resulting topology is invalid.
    pub fn remove_region(&mut self, id: RegionId) -> Result<RegionState, WorldSpatialError> {
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

    /// # Errors
    /// Returns [`WorldSpatialError`] when topology invariants are invalid.
    pub fn validate(&self) -> Result<(), WorldSpatialError> {
        match self.bounds {
            None => {
                if self.regions.is_empty() {
                    return Ok(());
                }
                return Err(WorldSpatialError::PartialInitialization);
            }
            Some(bounds) => {
                WorldBounds::new(
                    bounds.min_x_m,
                    bounds.max_x_m,
                    bounds.min_z_m,
                    bounds.max_z_m,
                )?;
            }
        }

        let bounds = self
            .bounds
            .ok_or(WorldSpatialError::PartialInitialization)?;
        if self.regions.is_empty() {
            return Err(WorldSpatialError::PartialInitialization);
        }

        for region in self.regions.iter() {
            validate_region_state(bounds, region)?;
            for &neighbor in &region.neighbors {
                if neighbor == region.id {
                    return Err(WorldSpatialError::SelfNeighbor { region: region.id });
                }
                let counterpart =
                    self.regions
                        .get(neighbor)
                        .ok_or(WorldSpatialError::UnknownNeighbor {
                            region: region.id,
                            neighbor,
                        })?;
                if counterpart.neighbors.binary_search(&region.id).is_err() {
                    return Err(WorldSpatialError::AsymmetricAdjacency {
                        region: region.id,
                        neighbor,
                    });
                }
            }
        }
        Ok(())
    }
}

fn validate_region_draft(
    bounds: WorldBounds,
    draft: &RegionDraft,
) -> Result<(), WorldSpatialError> {
    if !bounds.contains(draft.center) || draft.boundary.iter().any(|point| !bounds.contains(*point))
    {
        return Err(WorldSpatialError::PointOutsideBounds {
            region: RegionId(0),
        });
    }
    if draft.boundary.len() < 3 {
        return Err(WorldSpatialError::BoundaryTooShort {
            region: RegionId(0),
        });
    }
    validate_political(RegionId(0), draft.surface, draft.political)
}

fn validate_region_state(
    bounds: WorldBounds,
    region: &RegionState,
) -> Result<(), WorldSpatialError> {
    if region.id.0 == 0 {
        return Err(WorldSpatialError::InvalidRegionId { found: region.id });
    }
    if !bounds.contains(region.center)
        || region.boundary.iter().any(|point| !bounds.contains(*point))
    {
        return Err(WorldSpatialError::PointOutsideBounds { region: region.id });
    }
    if region.boundary.len() < 3 {
        return Err(WorldSpatialError::BoundaryTooShort { region: region.id });
    }
    if region.neighbors.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(WorldSpatialError::NonCanonicalNeighborOrder { region: region.id });
    }
    validate_political(region.id, region.surface, region.political)
}

fn validate_political(
    region: RegionId,
    surface: RegionSurface,
    political: RegionPoliticalState,
) -> Result<(), WorldSpatialError> {
    if surface == RegionSurface::Ocean
        && (political.legal_owner.is_some() || political.controller.is_some())
    {
        return Err(WorldSpatialError::OceanHasPoliticalOwner { region });
    }
    if political.controller.is_some() && political.legal_owner.is_none() {
        return Err(WorldSpatialError::ControllerWithoutOwner { region });
    }
    for country in [political.legal_owner, political.controller]
        .into_iter()
        .flatten()
    {
        if country.0 == 0 {
            return Err(WorldSpatialError::InvalidCountryId { region });
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldSpatialError {
    InvalidBounds,
    PartialInitialization,
    InvalidRegionRegistry,
    InvalidRegionId {
        found: RegionId,
    },
    BoundaryTooShort {
        region: RegionId,
    },
    PointOutsideBounds {
        region: RegionId,
    },
    SelfNeighbor {
        region: RegionId,
    },
    UnknownRegion {
        region: RegionId,
    },
    UnknownNeighbor {
        region: RegionId,
        neighbor: RegionId,
    },
    NonCanonicalNeighborOrder {
        region: RegionId,
    },
    AsymmetricAdjacency {
        region: RegionId,
        neighbor: RegionId,
    },
    OceanHasPoliticalOwner {
        region: RegionId,
    },
    ControllerWithoutOwner {
        region: RegionId,
    },
    InvalidCountryId {
        region: RegionId,
    },
}

impl core::fmt::Display for WorldSpatialError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidBounds => formatter.write_str("world bounds must have positive extent"),
            Self::PartialInitialization => {
                formatter.write_str("world spatial state is partially initialized")
            }
            Self::InvalidRegionRegistry => {
                formatter.write_str("Region registry allocator/state is invalid")
            }
            Self::InvalidRegionId { found } => {
                write!(formatter, "region id {} is invalid", found.0)
            }
            Self::BoundaryTooShort { region } => write!(
                formatter,
                "region {} boundary needs at least 3 points",
                region.0
            ),
            Self::PointOutsideBounds { region } => write!(
                formatter,
                "region {} has a point outside world bounds",
                region.0
            ),
            Self::SelfNeighbor { region } => write!(
                formatter,
                "region {} references itself as a neighbor",
                region.0
            ),
            Self::UnknownRegion { region } => {
                write!(formatter, "region {} does not exist", region.0)
            }
            Self::UnknownNeighbor { region, neighbor } => write!(
                formatter,
                "region {} references unknown neighbor {}",
                region.0, neighbor.0
            ),
            Self::NonCanonicalNeighborOrder { region } => write!(
                formatter,
                "region {} neighbor ids are not strictly ascending",
                region.0
            ),
            Self::AsymmetricAdjacency { region, neighbor } => write!(
                formatter,
                "region {} adjacency to {} is not reciprocal",
                region.0, neighbor.0
            ),
            Self::OceanHasPoliticalOwner { region } => write!(
                formatter,
                "ocean region {} cannot have legal ownership or control",
                region.0
            ),
            Self::ControllerWithoutOwner { region } => write!(
                formatter,
                "region {} cannot have a controller without a legal owner",
                region.0
            ),
            Self::InvalidCountryId { region } => {
                write!(formatter, "region {} references country id 0", region.0)
            }
        }
    }
}

impl std::error::Error for WorldSpatialError {}

#[cfg(test)]
mod tests {
    use super::{
        MapPoint, RegionDraft, RegionPoliticalState, RegionRegistry, RegionState, RegionSurface,
        WorldBounds, WorldSpatialState, TRIAL_REGION_COUNT,
    };
    use crate::{EntityRegistry, RegionId};

    fn line_regions(count: usize) -> Vec<RegionState> {
        (1..=count)
            .map(|index| {
                let id = RegionId(u64::try_from(index).expect("fixture id fits u64"));
                let x = i32::try_from(index).expect("fixture index fits i32") * 100;
                let mut neighbors = Vec::new();
                if index > 1 {
                    neighbors.push(RegionId(u64::try_from(index - 1).expect("id fits u64")));
                }
                if index < count {
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
            .collect()
    }

    fn bounds() -> WorldBounds {
        WorldBounds::new(0, 20_000, 0, 20_000).expect("bounds")
    }

    #[test]
    fn benchmark_count_is_not_registry_limit() {
        let benchmark =
            WorldSpatialState::new(bounds(), line_regions(TRIAL_REGION_COUNT)).expect("60");
        let larger = WorldSpatialState::new(bounds(), line_regions(75)).expect("75");
        assert_eq!(benchmark.regions.len(), 60);
        assert_eq!(larger.regions.len(), 75);
    }

    #[test]
    fn removed_region_id_is_not_reused_and_adjacency_is_cleaned() {
        let mut spatial = WorldSpatialState::new(bounds(), line_regions(3)).expect("topology");
        let removed = spatial.remove_region(RegionId(3)).expect("remove");
        assert_eq!(removed.id, RegionId(3));
        assert!(spatial.region(RegionId(3)).is_none());
        assert!(spatial
            .region(RegionId(2))
            .expect("region 2")
            .neighbors
            .is_empty());

        let created = spatial
            .create_region(RegionDraft {
                surface: RegionSurface::Land,
                center: MapPoint::new(400, 100),
                boundary: vec![
                    MapPoint::new(380, 80),
                    MapPoint::new(420, 80),
                    MapPoint::new(400, 120),
                ],
                neighbors: vec![RegionId(2)],
                political: RegionPoliticalState::unclaimed(),
            })
            .expect("create");
        assert_eq!(created, RegionId(4));
        assert_eq!(
            spatial.region(RegionId(2)).expect("region 2").neighbors,
            vec![RegionId(4)]
        );
    }

    #[test]
    fn persisted_registry_allocator_must_not_reuse_ids() {
        let records = line_regions(3);
        assert!(RegionRegistry::from_parts(3, records).is_err());
    }
}
