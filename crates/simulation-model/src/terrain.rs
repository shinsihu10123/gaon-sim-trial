#[path = "terrain_topology.rs"]
mod topology;

use crate::WorldBounds;

pub const TERRAIN_GRID_SIDE: u16 = 129;
pub const TERRAIN_SAMPLE_COUNT: usize = TERRAIN_GRID_SIDE as usize * TERRAIN_GRID_SIDE as usize;
pub const TERRAIN_GRID_SPACING_M: i32 = 10_000;
pub const TRIAL_WORLD_HALF_EXTENT_M: i32 = 640_000;
pub const TERRAIN_SEA_LEVEL_M: i16 = 0;
pub const NO_DOWNSTREAM_INDEX: u32 = u32::MAX;

#[must_use]
pub const fn trial_terrain_bounds() -> WorldBounds {
    WorldBounds {
        min_x_m: -TRIAL_WORLD_HALF_EXTENT_M,
        max_x_m: TRIAL_WORLD_HALF_EXTENT_M,
        min_z_m: -TRIAL_WORLD_HALF_EXTENT_M,
        max_z_m: TRIAL_WORLD_HALF_EXTENT_M,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReliefClass {
    DeepOcean,
    ShallowOcean,
    Coast,
    Plains,
    Hills,
    Mountains,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BiomeClass {
    Ocean,
    Grassland,
    Forest,
    Desert,
    Wetland,
    Alpine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TerrainSample {
    pub elevation_m: i16,
    pub moisture_permille: u16,
    pub relief: ReliefClass,
    pub biome: BiomeClass,
}

/// Deterministic drainage analysis derived from the canonical elevation field.
///
/// This object is intentionally not serialized independently: it is a pure
/// function of the authoritative heightfield and can always be regenerated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerrainHydrology {
    pub downstream_indices: Vec<u32>,
    pub drainage_basin_ids: Vec<u16>,
    pub flow_accumulation: Vec<u32>,
    pub river_orders: Vec<u8>,
}

/// Deterministic connected-land analysis derived from the canonical elevation field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerrainLandmasses {
    pub landmass_ids: Vec<u16>,
    pub island_landmass_ids: Vec<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerrainState {
    pub bounds: WorldBounds,
    pub width: u16,
    pub height: u16,
    pub spacing_m: i32,
    pub sea_level_m: i16,
    pub samples: Vec<TerrainSample>,
}

impl TerrainState {
    /// Creates the canonical TEST terrain grid.
    ///
    /// # Errors
    ///
    /// Returns [`TerrainError`] if dimensions, bounds, sample counts or terrain
    /// classification invariants are invalid.
    pub fn new_trial(samples: Vec<TerrainSample>) -> Result<Self, TerrainError> {
        let terrain = Self {
            bounds: trial_terrain_bounds(),
            width: TERRAIN_GRID_SIDE,
            height: TERRAIN_GRID_SIDE,
            spacing_m: TERRAIN_GRID_SPACING_M,
            sea_level_m: TERRAIN_SEA_LEVEL_M,
            samples,
        };
        terrain.validate()?;
        Ok(terrain)
    }

    #[must_use]
    pub fn index(&self, x: u16, z: u16) -> Option<usize> {
        if x >= self.width || z >= self.height {
            return None;
        }
        Some(usize::from(z) * usize::from(self.width) + usize::from(x))
    }

    #[must_use]
    pub fn sample(&self, x: u16, z: u16) -> Option<&TerrainSample> {
        self.index(x, z).and_then(|index| self.samples.get(index))
    }

    #[must_use]
    pub fn land_sample_count(&self) -> usize {
        self.samples
            .iter()
            .filter(|sample| sample.elevation_m >= self.sea_level_m)
            .count()
    }

    /// Derives the canonical drainage graph, basin IDs and river hierarchy.
    #[must_use]
    pub fn derive_hydrology(&self) -> TerrainHydrology {
        topology::derive_terrain_hydrology(self)
    }

    /// Derives connected landmasses and the set of non-mainland island IDs.
    #[must_use]
    pub fn derive_landmasses(&self) -> TerrainLandmasses {
        topology::derive_terrain_landmasses(self)
    }

    /// Validates the fixed TEST terrain representation.
    ///
    /// # Errors
    ///
    /// Returns [`TerrainError`] when any canonical dimension or sample
    /// classification invariant is violated.
    pub fn validate(&self) -> Result<(), TerrainError> {
        if self.bounds != trial_terrain_bounds()
            || self.width != TERRAIN_GRID_SIDE
            || self.height != TERRAIN_GRID_SIDE
            || self.spacing_m != TERRAIN_GRID_SPACING_M
            || self.sea_level_m != TERRAIN_SEA_LEVEL_M
        {
            return Err(TerrainError::NonCanonicalGrid);
        }
        if self.samples.len() != TERRAIN_SAMPLE_COUNT {
            return Err(TerrainError::WrongSampleCount {
                found: self.samples.len(),
            });
        }

        for (index, sample) in self.samples.iter().enumerate() {
            if sample.moisture_permille > 1_000 {
                return Err(TerrainError::InvalidMoisture { index });
            }

            let underwater = sample.elevation_m < self.sea_level_m;
            let ocean_relief = matches!(
                sample.relief,
                ReliefClass::DeepOcean | ReliefClass::ShallowOcean
            );
            let ocean_biome = sample.biome == BiomeClass::Ocean;

            if underwater != ocean_relief || underwater != ocean_biome {
                return Err(TerrainError::ClassificationMismatch { index });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainError {
    NonCanonicalGrid,
    WrongSampleCount { found: usize },
    InvalidMoisture { index: usize },
    ClassificationMismatch { index: usize },
}

impl core::fmt::Display for TerrainError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NonCanonicalGrid => {
                formatter.write_str("terrain grid does not match TEST geometry")
            }
            Self::WrongSampleCount { found } => write!(
                formatter,
                "terrain requires {TERRAIN_SAMPLE_COUNT} samples, found {found}"
            ),
            Self::InvalidMoisture { index } => write!(
                formatter,
                "terrain sample {index} moisture exceeds 1000 permille"
            ),
            Self::ClassificationMismatch { index } => write!(
                formatter,
                "terrain sample {index} elevation and classification disagree"
            ),
        }
    }
}

impl std::error::Error for TerrainError {}

#[cfg(test)]
mod tests {
    use super::{
        BiomeClass, ReliefClass, TerrainError, TerrainSample, TerrainState, TERRAIN_SAMPLE_COUNT,
    };

    fn ocean_samples() -> Vec<TerrainSample> {
        vec![
            TerrainSample {
                elevation_m: -500,
                moisture_permille: 500,
                relief: ReliefClass::ShallowOcean,
                biome: BiomeClass::Ocean,
            };
            TERRAIN_SAMPLE_COUNT
        ]
    }

    #[test]
    fn canonical_grid_accepts_exact_sample_count() {
        let terrain = TerrainState::new_trial(ocean_samples()).expect("terrain should validate");
        assert_eq!(terrain.samples.len(), TERRAIN_SAMPLE_COUNT);
        assert_eq!(terrain.land_sample_count(), 0);
        assert!(terrain.derive_landmasses().island_landmass_ids.is_empty());
    }

    #[test]
    fn incorrect_sample_count_is_rejected() {
        let mut samples = ocean_samples();
        samples.pop();
        assert!(matches!(
            TerrainState::new_trial(samples),
            Err(TerrainError::WrongSampleCount { .. })
        ));
    }

    #[test]
    fn underwater_sample_requires_ocean_classes() {
        let mut samples = ocean_samples();
        samples[0].relief = ReliefClass::Plains;
        assert!(matches!(
            TerrainState::new_trial(samples),
            Err(TerrainError::ClassificationMismatch { index: 0 })
        ));
    }
}
