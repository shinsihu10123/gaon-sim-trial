use crate::{CountryId, RegionId};

/// Fixed region count for the TEST v0.1 world.
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
///
/// Detailed elevation, biome, rivers, resources and infrastructure are added by
/// later Stage 2 subsections; this type only establishes topology and political
/// identity needed to attach those systems without changing `RegionId` semantics.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RegionState {
    pub id: RegionId,
    pub surface: RegionSurface,
    pub center: MapPoint,
    /// Ordered polygon ring. The first point is not repeated at the end.
    pub boundary: Vec<MapPoint>,
    /// Strictly ascending, unique `RegionIds`. Adjacency must be reciprocal.
    pub neighbors: Vec<RegionId>,
    pub political: RegionPoliticalState,
}

/// Spatial layer of the authoritative world.
///
/// Stage 1 engines start with `uninitialized()`. Once world generation runs in
/// Stage 2, the state becomes a validated 60-region continuous map.
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

    /// Constructs the canonical TEST world topology.
    ///
    /// # Errors
    ///
    /// Returns [`WorldSpatialError`] when region count, IDs, geometry, political
    /// ownership or adjacency invariants are violated.
    pub fn new_trial(
        bounds: WorldBounds,
        regions: Vec<RegionState>,
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
        if id.0 == 0 {
            return None;
        }
        self.regions.get(usize::from(id.0 - 1))
    }

    /// Validates canonical Stage 2.1 topology.
    ///
    /// An entirely empty `(None, [])` state is the only valid uninitialized
    /// representation. Any initialized representation must contain exactly 60
    /// regions with IDs 1..=60 in vector order.
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

        if self.regions.len() != TRIAL_REGION_COUNT {
            return Err(WorldSpatialError::WrongRegionCount {
                found: self.regions.len(),
            });
        }

        let Some(bounds) = self.bounds else {
            return Err(WorldSpatialError::PartialInitialization);
        };

        for (index, region) in self.regions.iter().enumerate() {
            let Ok(expected) = u16::try_from(index + 1) else {
                return Err(WorldSpatialError::WrongRegionCount {
                    found: self.regions.len(),
                });
            };
            if region.id != RegionId(expected) {
                return Err(WorldSpatialError::NonCanonicalRegionId {
                    expected: RegionId(expected),
                    found: region.id,
                });
            }

            if !bounds.contains(region.center) {
                return Err(WorldSpatialError::PointOutsideBounds { region: region.id });
            }
            if region.boundary.len() < 3 {
                return Err(WorldSpatialError::BoundaryTooShort { region: region.id });
            }
            if region.boundary.iter().any(|&point| !bounds.contains(point)) {
                return Err(WorldSpatialError::PointOutsideBounds { region: region.id });
            }

            let mut previous = None;
            for &neighbor in &region.neighbors {
                if neighbor == region.id {
                    return Err(WorldSpatialError::SelfNeighbor { region: region.id });
                }
                if neighbor.0 == 0 || usize::from(neighbor.0) > TRIAL_REGION_COUNT {
                    return Err(WorldSpatialError::UnknownNeighbor {
                        region: region.id,
                        neighbor,
                    });
                }
                if previous.is_some_and(|previous| previous >= neighbor) {
                    return Err(WorldSpatialError::NonCanonicalNeighborOrder { region: region.id });
                }
                previous = Some(neighbor);
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
    WrongRegionCount {
        found: usize,
    },
    NonCanonicalRegionId {
        expected: RegionId,
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
            Self::WrongRegionCount { found } => write!(
                formatter,
                "trial world requires {TRIAL_REGION_COUNT} regions, found {found}"
            ),
            Self::NonCanonicalRegionId { expected, found } => write!(
                formatter,
                "expected region id {}, found {}",
                expected.0, found.0
            ),
            Self::BoundaryTooShort { region } => {
                write!(
                    formatter,
                    "region {} boundary needs at least 3 points",
                    region.0
                )
            }
            Self::PointOutsideBounds { region } => {
                write!(
                    formatter,
                    "region {} has a point outside world bounds",
                    region.0
                )
            }
            Self::SelfNeighbor { region } => {
                write!(
                    formatter,
                    "region {} references itself as a neighbor",
                    region.0
                )
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
                "ocean region {} cannot have legal ownership or control in Stage 2.1",
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

    fn trial_regions() -> Vec<RegionState> {
        (1..=TRIAL_REGION_COUNT)
            .map(|index| {
                let id = RegionId(u16::try_from(index).expect("trial id fits u16"));
                let x = i32::try_from(index).expect("trial index fits i32") * 100;
                let mut neighbors = Vec::new();
                if index > 1 {
                    neighbors.push(RegionId(u16::try_from(index - 1).expect("id fits u16")));
                }
                if index < TRIAL_REGION_COUNT {
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

    #[test]
    fn uninitialized_world_has_one_canonical_representation() {
        let spatial = WorldSpatialState::uninitialized();
        assert!(!spatial.is_initialized());
        assert!(spatial.validate().is_ok());
        assert_eq!(spatial.region(RegionId(1)), None);
    }

    #[test]
    fn canonical_sixty_region_topology_validates() {
        let bounds = WorldBounds::new(0, 10_000, 0, 10_000).expect("bounds are valid");
        let spatial = WorldSpatialState::new_trial(bounds, trial_regions())
            .expect("canonical trial topology should validate");
        assert!(spatial.is_initialized());
        assert_eq!(spatial.regions.len(), 60);
        assert_eq!(
            spatial.region(RegionId(60)).expect("region exists").id,
            RegionId(60)
        );
    }

    #[test]
    fn adjacency_must_be_reciprocal() {
        let bounds = WorldBounds::new(0, 10_000, 0, 10_000).expect("bounds are valid");
        let mut regions = trial_regions();
        regions[1].neighbors.clear();
        assert!(matches!(
            WorldSpatialState::new_trial(bounds, regions),
            Err(WorldSpatialError::AsymmetricAdjacency { .. })
        ));
    }

    #[test]
    fn legal_owner_and_controller_are_distinct_fields() {
        let bounds = WorldBounds::new(0, 10_000, 0, 10_000).expect("bounds are valid");
        let mut regions = trial_regions();
        regions[0].political = RegionPoliticalState {
            legal_owner: Some(CountryId(1)),
            controller: Some(CountryId(2)),
        };
        let spatial = WorldSpatialState::new_trial(bounds, regions)
            .expect("occupation-style owner/controller split is valid");
        let first = spatial.region(RegionId(1)).expect("region exists");
        assert_eq!(first.political.legal_owner, Some(CountryId(1)));
        assert_eq!(first.political.controller, Some(CountryId(2)));
    }

    #[test]
    fn ocean_region_cannot_be_owned_in_stage_two_one() {
        let bounds = WorldBounds::new(0, 10_000, 0, 10_000).expect("bounds are valid");
        let mut regions = trial_regions();
        regions[0].surface = RegionSurface::Ocean;
        regions[0].political = RegionPoliticalState::sovereign(CountryId(1));
        assert!(matches!(
            WorldSpatialState::new_trial(bounds, regions),
            Err(WorldSpatialError::OceanHasPoliticalOwner {
                region: RegionId(1)
            })
        ));
    }
}
