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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerrainHydrology {
    pub downstream_indices: Vec<u32>,
    pub drainage_basin_ids: Vec<u16>,
    pub flow_accumulation: Vec<u32>,
    pub river_orders: Vec<u8>,
}

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
    pub hydrology: Option<TerrainHydrology>,
    pub landmasses: Option<TerrainLandmasses>,
}

impl TerrainState {
    /// Creates the canonical TEST terrain grid without derived hydrology.
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
            hydrology: None,
            landmasses: None,
        };
        terrain.validate()?;
        Ok(terrain)
    }

    /// Attaches deterministic drainage and landmass topology to a terrain.
    ///
    /// # Errors
    ///
    /// Returns [`TerrainError`] when the derived arrays are incomplete or
    /// inconsistent with the elevation field.
    pub fn with_derived_geography(
        mut self,
        hydrology: TerrainHydrology,
        landmasses: TerrainLandmasses,
    ) -> Result<Self, TerrainError> {
        self.hydrology = Some(hydrology);
        self.landmasses = Some(landmasses);
        self.validate()?;
        Ok(self)
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

    #[must_use]
    pub fn river_sample_count(&self) -> usize {
        self.hydrology.as_ref().map_or(0, |hydrology| {
            hydrology
                .river_orders
                .iter()
                .filter(|&&order| order > 0)
                .count()
        })
    }

    #[must_use]
    pub fn island_count(&self) -> usize {
        self.landmasses
            .as_ref()
            .map_or(0, |landmasses| landmasses.island_landmass_ids.len())
    }

    /// Validates the fixed TEST terrain representation.
    ///
    /// # Errors
    ///
    /// Returns [`TerrainError`] when any canonical dimension, sample,
    /// hydrology or landmass invariant is violated.
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

        match (&self.hydrology, &self.landmasses) {
            (None, None) => Ok(()),
            (Some(hydrology), Some(landmasses)) => {
                self.validate_hydrology(hydrology)?;
                self.validate_landmasses(landmasses)
            }
            _ => Err(TerrainError::PartialDerivedGeography),
        }
    }

    fn validate_hydrology(&self, hydrology: &TerrainHydrology) -> Result<(), TerrainError> {
        for (field, found) in [
            ("downstream_indices", hydrology.downstream_indices.len()),
            ("drainage_basin_ids", hydrology.drainage_basin_ids.len()),
            ("flow_accumulation", hydrology.flow_accumulation.len()),
            ("river_orders", hydrology.river_orders.len()),
        ] {
            if found != TERRAIN_SAMPLE_COUNT {
                return Err(TerrainError::WrongDerivedFieldLength { field, found });
            }
        }

        for index in 0..TERRAIN_SAMPLE_COUNT {
            let sample = self.samples[index];
            let downstream = hydrology.downstream_indices[index];
            let basin = hydrology.drainage_basin_ids[index];
            let flow = hydrology.flow_accumulation[index];
            let river_order = hydrology.river_orders[index];

            if sample.elevation_m < self.sea_level_m {
                if downstream != NO_DOWNSTREAM_INDEX || basin != 0 || flow != 0 || river_order != 0 {
                    return Err(TerrainError::InvalidOceanHydrology { index });
                }
                continue;
            }

            if basin == 0 || flow == 0 || river_order > 3 {
                return Err(TerrainError::InvalidLandHydrology { index });
            }
            if downstream != NO_DOWNSTREAM_INDEX {
                let downstream_index = usize::try_from(downstream)
                    .map_err(|_| TerrainError::InvalidDownstream { index })?;
                let downstream_sample = self
                    .samples
                    .get(downstream_index)
                    .ok_or(TerrainError::InvalidDownstream { index })?;
                if downstream_sample.elevation_m >= sample.elevation_m {
                    return Err(TerrainError::InvalidDownstream { index });
                }
            }
        }
        Ok(())
    }

    fn validate_landmasses(&self, landmasses: &TerrainLandmasses) -> Result<(), TerrainError> {
        if landmasses.landmass_ids.len() != TERRAIN_SAMPLE_COUNT {
            return Err(TerrainError::WrongDerivedFieldLength {
                field: "landmass_ids",
                found: landmasses.landmass_ids.len(),
            });
        }

        for (index, (&landmass_id, sample)) in landmasses
            .landmass_ids
            .iter()
            .zip(&self.samples)
            .enumerate()
        {
            let land = sample.elevation_m >= self.sea_level_m;
            if land != (landmass_id > 0) {
                return Err(TerrainError::InvalidLandmass { index });
            }
        }

        let mut previous = 0_u16;
        for &island_id in &landmasses.island_landmass_ids {
            if island_id == 0 || island_id <= previous || !landmasses.landmass_ids.contains(&island_id) {
                return Err(TerrainError::InvalidIslandList);
            }
            previous = island_id;
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
    PartialDerivedGeography,
    WrongDerivedFieldLength { field: &'static str, found: usize },
    InvalidOceanHydrology { index: usize },
    InvalidLandHydrology { index: usize },
    InvalidDownstream { index: usize },
    InvalidLandmass { index: usize },
    InvalidIslandList,
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
            Self::PartialDerivedGeography => {
                formatter.write_str("terrain hydrology and landmass topology must be attached together")
            }
            Self::WrongDerivedFieldLength { field, found } => write!(
                formatter,
                "terrain derived field {field} requires {TERRAIN_SAMPLE_COUNT} entries, found {found}"
            ),
            Self::InvalidOceanHydrology { index } => {
                write!(formatter, "ocean terrain sample {index} contains land hydrology")
            }
            Self::InvalidLandHydrology { index } => {
                write!(formatter, "land terrain sample {index} has invalid hydrology")
            }
            Self::InvalidDownstream { index } => {
                write!(formatter, "terrain sample {index} has invalid downstream target")
            }
            Self::InvalidLandmass { index } => {
                write!(formatter, "terrain sample {index} has invalid landmass membership")
            }
            Self::InvalidIslandList => formatter.write_str("terrain island landmass IDs are invalid"),
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
