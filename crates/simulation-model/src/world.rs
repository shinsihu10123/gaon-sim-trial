use crate::{CountryId, RegionId};

/// Region count used by the Standard Benchmark S fixture.
///
/// This constant is a benchmark preset only. The simulation core does not
/// require an initialized world to contain this number of regions.
pub const TRIAL_REGION_COUNT: usize = 60;

/// Authoritative horizontal map coordinate in integer metres.
///
/// Rendering may convert this to floating point, but simulation/save state keeps
/// integer coordinates so geometry keys are deterministic across platforms.
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

/// Rectangular bounds of the continuous simulation world.
///
/// Bounds are independent from the number of regions. Region generation may
/// choose any validated partition inside these coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorldBounds {
    pub min_x_m: i32,
    pub max_x_m: i32,
    pub min_z_m: i32,
    pub max_z_m: i32,
}

impl WorldBounds {
    /// Creates non-empty world bounds.
    ///
    /// # Errors
    ///
    /// Returns [`WorldSpatialError::InvalidBounds`] when either axis has zero or
    /// negative extent.
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

/// Coarse region surface classification required before detailed terrain exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegionSurface {
    Land,
    Ocean,
}

/// Legal and effective control are intentionally separate.
///
/// `legal_owner` is treaty-recognized sovereignty. `controller` is the current
/// administrative/military controller and may differ during occupation.
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

/// Canonical region record shared by simulation, persistence and render protocol.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RegionState {
    pub id: RegionId,
    pub surface: RegionSurface,
    pub center: MapPoint,
    /// Ordered polygon ring. The first point is not repeated at the end.
    pub boundary: Vec<MapPoint>,
    /// Strictly ascending, unique `RegionId`s. Adjacency must be reciprocal.
    pub neighbors: Vec<RegionId>,
    pub political: RegionPoliticalState,
}

/// Spatial layer of the authoritative world.
///
/// Regions are stored in strictly ascending ID order. IDs need not be contiguous,
/// so creation/removal can later be handled by the Dynamic Entity Architecture
/// without renumbering surviving regions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldSpatialState {
    pub bounds: Option<WorldBounds>,
    pub regions: Vec<RegionState>,
}

impl WorldSpatialState {
    #[must_use]
    pub const fn uninitialized() -> Self {
        Self {
            bounds: None,
            regions: Vec::new(),
        }
    }

    /// Constructs a canonical spatial topology with a variable region count.
    ///
    /// # Errors
    ///
    /// Returns [`WorldSpatialError`] when IDs, geometry, political ownership or
    /// adjacency invariants are violated.
    pub fn new(bounds: WorldBounds, regions: Vec<RegionState>) -> Result<Self, WorldSpatialError> {
        let spatial = Self {
            bounds: Some(bounds),
            regions,
        };
        spatial.validate()?;
        Ok(spatial)
    }

    /// Backward-compatible constructor retained for benchmark fixtures.
    ///
    /// This does not enforce [`TRIAL_REGION_COUNT`]; the benchmark caller chooses
    /// the count explicitly.
    pub fn new_trial(
        bounds: WorldBounds,
        regions: Vec<RegionState>,
    ) -> Result<Self, WorldSpatialError> {
        Self::new(bounds, regions)
    }

    #[must_use]
    pub fn is_initialized(&self) -> bool {
        self.bounds.is_some()
    }

    /// Looks up a region by stable ID without assuming contiguous identifiers.
    #[must_use]
    pub fn region(&self, id: RegionId) -> Option<&RegionState> {
        if id.0 == 0 {
            return None;
        }
        self.regions
            .binary_search_by_key(&id, |region| region.id)
            .ok()
            .and_then(|index| self.regions.get(index))
    }

    /// Validates canonical dynamic-region topology.
    ///
    /// An entirely empty `(None, [])` state is the only uninitialized form.
    /// Initialized states may contain any region count representable by the
    /// collection. Region IDs must be non-zero, unique and strictly ascending,
    /// but are not required to be contiguous.
    ///
    /// # Errors
    ///
    /// Returns [`WorldSpatialError`] when initialization, region identity,
    /// geometry, adjacency or political invariants are invalid.
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

        let Some(bounds) = self.bounds else {
            return Err(WorldSpatialError::PartialInitialization);
        };

        let mut previous_region_id = None;
        for region in &self.regions {
            if region.id.0 == 0 {
                return Err(WorldSpatialError::InvalidRegionId { found: region.id });
            }
            if let Some(previous) = previous_region_id {
                if previous >= region.id {
                    return Err(WorldSpatialError::NonCanonicalRegionOrder {
                        previous,
                        found: region.id,
                    });
                }
            }
            previous_region_id = Some(region.id);

            if !bounds.contains(region.center) {
                return Err(WorldSpatialError::PointOutsideBounds { region: region.id });
            }
            if region.boundary.len() < 3 {
                return Err(WorldSpatialError::BoundaryTooShort { region: region.id });
            }
            if region.boundary.iter().any(|&point| !bounds.contains(point)) {
                return Err(WorldSpatialError::PointOutsideBounds { region: region.id });
            }

            let mut previous_neighbor = None;
            for &neighbor in &region.neighbors {
                if neighbor == region.id {
                    return Err(WorldSpatialError::SelfNeighbor { region: region.id });
                }
                if neighbor.0 == 0 || self.region(neighbor).is_none() {
                    return Err(WorldSpatialError::UnknownNeighbor {
                        region: region.id,
                        neighbor,
                    });
                }
                if previous_neighbor.is_some_and(|previous| previous >= neighbor) {
                    return Err(WorldSpatialError::NonCanonicalNeighborOrder { region: region.id });
                }
                previous_neighbor = Some(neighbor);
            }

            if region.surface == RegionSurface::Ocean
                && (region.political.legal_owner.is_some() || region.political.controller.is_some())
            {
                return Err(WorldSpatialError::OceanHasPoliticalOwner { region: region.id });
            }
            if region.political.controller.is_some() && region.political.legal_owner.is_none() {
                return Err(WorldSpatialError::ControllerWithoutOwner { region: region.id });
            }
            for country in [region.political.legal_owner, region.political.controller]
                .into_iter()
                .flatten()
            {
                if country.0 == 0 {
                    return Err(WorldSpatialError::InvalidCountryId { region: region.id });
                }
            }
        }

        for region in &self.regions {
            for &neighbor in &region.neighbors {
                let counterpart =
                    self.region(neighbor)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldSpatialError {
    InvalidBounds,
    PartialInitialization,
    InvalidRegionId {
        found: RegionId,
    },
    NonCanonicalRegionOrder {
        previous: RegionId,
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
                formatter.write_str("world spatial state is only partially initialized")
            }
            Self::InvalidRegionId { found } => {
                write!(formatter, "region id {} is invalid", found.0)
            }
            Self::NonCanonicalRegionOrder { previous, found } => write!(
                formatter,
                "region ids must be strictly ascending: {} before {}",
                previous.0, found.0
            ),
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
        MapPoint, RegionPoliticalState, RegionState, RegionSurface, WorldBounds, WorldSpatialError,
        WorldSpatialState, TRIAL_REGION_COUNT,
    };
    use crate::{CountryId, RegionId};

    fn line_regions(count: usize) -> Vec<RegionState> {
        (1..=count)
            .map(|index| {
                let id = RegionId(u16::try_from(index).expect("fixture id fits u16"));
                let x = i32::try_from(index).expect("fixture index fits i32") * 100;
                let mut neighbors = Vec::new();
                if index > 1 {
                    neighbors.push(RegionId(u16::try_from(index - 1).expect("id fits u16")));
                }
                if index < count {
                    neighbors.push(RegionId(u16::try_from(index + 1).expect("id fits u16")));
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
        WorldBounds::new(0, 20_000, 0, 20_000).expect("bounds are valid")
    }

    #[test]
    fn uninitialized_world_has_one_canonical_representation() {
        let spatial = WorldSpatialState::uninitialized();
        assert!(!spatial.is_initialized());
        assert!(spatial.validate().is_ok());
        assert_eq!(spatial.region(RegionId(1)), None);
    }

    #[test]
    fn benchmark_sixty_region_topology_still_validates() {
        let spatial = WorldSpatialState::new(bounds(), line_regions(TRIAL_REGION_COUNT))
            .expect("benchmark topology should validate");
        assert_eq!(spatial.regions.len(), TRIAL_REGION_COUNT);
        assert_eq!(
            spatial.region(RegionId(60)).map(|region| region.id),
            Some(RegionId(60))
        );
    }

    #[test]
    fn variable_region_counts_validate() {
        for count in [1, 3, 75] {
            let spatial = WorldSpatialState::new(bounds(), line_regions(count))
                .expect("region count must not be a core invariant");
            assert_eq!(spatial.regions.len(), count);
        }
    }

    #[test]
    fn sparse_region_ids_are_valid_and_lookup_is_id_based() {
        let regions = vec![
            RegionState {
                id: RegionId(10),
                surface: RegionSurface::Land,
                center: MapPoint::new(100, 100),
                boundary: vec![
                    MapPoint::new(80, 80),
                    MapPoint::new(120, 80),
                    MapPoint::new(100, 120),
                ],
                neighbors: vec![RegionId(20)],
                political: RegionPoliticalState::unclaimed(),
            },
            RegionState {
                id: RegionId(20),
                surface: RegionSurface::Land,
                center: MapPoint::new(200, 100),
                boundary: vec![
                    MapPoint::new(180, 80),
                    MapPoint::new(220, 80),
                    MapPoint::new(200, 120),
                ],
                neighbors: vec![RegionId(10), RegionId(40)],
                political: RegionPoliticalState::unclaimed(),
            },
            RegionState {
                id: RegionId(40),
                surface: RegionSurface::Land,
                center: MapPoint::new(300, 100),
                boundary: vec![
                    MapPoint::new(280, 80),
                    MapPoint::new(320, 80),
                    MapPoint::new(300, 120),
                ],
                neighbors: vec![RegionId(20)],
                political: RegionPoliticalState::unclaimed(),
            },
        ];
        let spatial = WorldSpatialState::new(bounds(), regions).expect("sparse IDs are canonical");
        assert_eq!(
            spatial.region(RegionId(20)).map(|region| region.id),
            Some(RegionId(20))
        );
        assert!(spatial.region(RegionId(30)).is_none());
    }

    #[test]
    fn region_ids_must_be_strictly_sorted() {
        let mut regions = line_regions(3);
        regions.swap(0, 1);
        assert!(matches!(
            WorldSpatialState::new(bounds(), regions),
            Err(WorldSpatialError::NonCanonicalRegionOrder { .. })
        ));
    }

    #[test]
    fn adjacency_must_reference_existing_regions() {
        let mut regions = line_regions(3);
        regions[2].neighbors.push(RegionId(60));
        assert!(matches!(
            WorldSpatialState::new(bounds(), regions),
            Err(WorldSpatialError::UnknownNeighbor {
                region: RegionId(3),
                neighbor: RegionId(60)
            })
        ));
    }

    #[test]
    fn adjacency_must_be_reciprocal() {
        let mut regions = line_regions(3);
        regions[1].neighbors.clear();
        assert!(matches!(
            WorldSpatialState::new(bounds(), regions),
            Err(WorldSpatialError::AsymmetricAdjacency { .. })
        ));
    }

    #[test]
    fn legal_owner_and_controller_are_distinct_fields() {
        let mut regions = line_regions(3);
        regions[0].political = RegionPoliticalState {
            legal_owner: Some(CountryId(1)),
            controller: Some(CountryId(2)),
        };
        let spatial = WorldSpatialState::new(bounds(), regions)
            .expect("occupation-style owner/controller split is valid");
        let first = spatial.region(RegionId(1)).expect("region exists");
        assert_eq!(first.political.legal_owner, Some(CountryId(1)));
        assert_eq!(first.political.controller, Some(CountryId(2)));
    }

    #[test]
    fn ocean_region_cannot_be_owned() {
        let mut regions = line_regions(3);
        regions[0].surface = RegionSurface::Ocean;
        regions[0].political = RegionPoliticalState::sovereign(CountryId(1));
        assert!(matches!(
            WorldSpatialState::new(bounds(), regions),
            Err(WorldSpatialError::OceanHasPoliticalOwner {
                region: RegionId(1)
            })
        ));
    }
}
