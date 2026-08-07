use crate::{TerrainEffectField, TerrainState, TERRAIN_SAMPLE_COUNT};

/// Finite extractive resource stock attached to one canonical terrain sample.
///
/// Quantities use TEST accounting units specific to each resource family:
/// energy uses GWh-equivalent, metals and construction resources use
/// kilotonne-equivalent. Quality and accessibility are integer permille.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceDeposit {
    pub initial_quantity: u64,
    pub remaining_quantity: u64,
    pub quality_permille: u16,
    pub accessibility_permille: u16,
}

impl ResourceDeposit {
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            initial_quantity: 0,
            remaining_quantity: 0,
            quality_permille: 0,
            accessibility_permille: 0,
        }
    }

    /// Constructs a finite resource deposit.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError`] if quality or accessibility exceeds 1000.
    pub fn new(
        quantity: u64,
        quality_permille: u16,
        accessibility_permille: u16,
    ) -> Result<Self, ResourceError> {
        if quality_permille > 1_000 {
            return Err(ResourceError::InvalidQuality(quality_permille));
        }
        if accessibility_permille > 1_000 {
            return Err(ResourceError::InvalidAccessibility(accessibility_permille));
        }
        if quantity == 0 {
            return Ok(Self::empty());
        }
        Ok(Self {
            initial_quantity: quantity,
            remaining_quantity: quantity,
            quality_permille,
            accessibility_permille,
        })
    }

    #[must_use]
    pub fn depletion_permille(self) -> u16 {
        if self.initial_quantity == 0 {
            return 0;
        }
        let depleted = self
            .initial_quantity
            .saturating_sub(self.remaining_quantity);
        let value = depleted.saturating_mul(1_000) / self.initial_quantity;
        u16::try_from(value).unwrap_or(1_000)
    }

    /// Extracts at most the remaining quantity and mutates authoritative stock.
    #[must_use]
    pub fn extract(&mut self, requested_quantity: u64) -> ExtractionResult {
        let extracted_quantity = requested_quantity.min(self.remaining_quantity);
        self.remaining_quantity -= extracted_quantity;
        ExtractionResult {
            requested_quantity,
            extracted_quantity,
            remaining_quantity: self.remaining_quantity,
        }
    }

    /// Validates persisted deposit invariants.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError`] if a persisted stock is impossible.
    pub fn validate(self) -> Result<(), ResourceError> {
        if self.quality_permille > 1_000 {
            return Err(ResourceError::InvalidQuality(self.quality_permille));
        }
        if self.accessibility_permille > 1_000 {
            return Err(ResourceError::InvalidAccessibility(
                self.accessibility_permille,
            ));
        }
        if self.remaining_quantity > self.initial_quantity {
            return Err(ResourceError::RemainingExceedsInitial);
        }
        if self.initial_quantity == 0
            && (self.remaining_quantity != 0
                || self.quality_permille != 0
                || self.accessibility_permille != 0)
        {
            return Err(ResourceError::InvalidEmptyDeposit);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExtractionResult {
    pub requested_quantity: u64,
    pub extracted_quantity: u64,
    pub remaining_quantity: u64,
}

/// Resource endowment aligned one-to-one with a canonical terrain sample.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceCellState {
    /// Renewable geography-only food production base in tonnes/year before
    /// technology, labor, capital and agricultural infrastructure.
    pub food_capacity_tonnes_per_year: u32,
    /// Finite energy reserve in GWh-equivalent TEST accounting units.
    pub energy: ResourceDeposit,
    /// Finite metal reserve in kilotonne-equivalent TEST accounting units.
    pub metals: ResourceDeposit,
    /// Combined timber/mineral construction reserve in kilotonne-equivalent.
    pub construction: ResourceDeposit,
}

impl ResourceCellState {
    pub const OCEAN: Self = Self {
        food_capacity_tonnes_per_year: 0,
        energy: ResourceDeposit::empty(),
        metals: ResourceDeposit::empty(),
        construction: ResourceDeposit::empty(),
    };

    /// Validates all resource stocks in the cell.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError`] if a finite deposit is invalid.
    pub fn validate(self) -> Result<(), ResourceError> {
        self.energy.validate()?;
        self.metals.validate()?;
        self.construction.validate()
    }
}

/// Authoritative Stage 2.4 resource field. Cell order is exactly the canonical
/// terrain sample order, allowing deterministic later aggregation into regions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceFieldState {
    pub cells: Vec<ResourceCellState>,
}

impl ResourceFieldState {
    /// Builds and validates a resource field.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError`] when the field is not aligned with the TEST
    /// heightfield or any finite stock is invalid.
    pub fn new(cells: Vec<ResourceCellState>) -> Result<Self, ResourceError> {
        if cells.len() != TERRAIN_SAMPLE_COUNT {
            return Err(ResourceError::WrongCellCount { found: cells.len() });
        }
        for (index, cell) in cells.iter().copied().enumerate() {
            cell.validate()
                .map_err(|source| ResourceError::InvalidCell {
                    index,
                    source: source.kind(),
                })?;
        }
        Ok(Self { cells })
    }

    #[must_use]
    pub fn cell(&self, index: usize) -> Option<&ResourceCellState> {
        self.cells.get(index)
    }

    #[must_use]
    pub fn cell_mut(&mut self, index: usize) -> Option<&mut ResourceCellState> {
        self.cells.get_mut(index)
    }

    /// Aggregates resource endowments for a later region assignment.
    #[must_use]
    pub fn aggregate_indices(&self, indices: &[usize]) -> Option<ResourceAggregate> {
        if indices.is_empty() {
            return None;
        }
        let mut aggregate = ResourceAggregate::default();
        let mut energy_quality_weighted = 0_u128;
        let mut energy_access_weighted = 0_u128;
        let mut metal_quality_weighted = 0_u128;
        let mut metal_access_weighted = 0_u128;
        let mut construction_quality_weighted = 0_u128;
        let mut construction_access_weighted = 0_u128;

        for &index in indices {
            let cell = self.cells.get(index)?;
            aggregate.food_capacity_tonnes_per_year = aggregate
                .food_capacity_tonnes_per_year
                .saturating_add(u64::from(cell.food_capacity_tonnes_per_year));
            aggregate.energy_remaining = aggregate
                .energy_remaining
                .saturating_add(cell.energy.remaining_quantity);
            aggregate.metals_remaining = aggregate
                .metals_remaining
                .saturating_add(cell.metals.remaining_quantity);
            aggregate.construction_remaining = aggregate
                .construction_remaining
                .saturating_add(cell.construction.remaining_quantity);
            energy_quality_weighted += u128::from(cell.energy.remaining_quantity)
                * u128::from(cell.energy.quality_permille);
            energy_access_weighted += u128::from(cell.energy.remaining_quantity)
                * u128::from(cell.energy.accessibility_permille);
            metal_quality_weighted += u128::from(cell.metals.remaining_quantity)
                * u128::from(cell.metals.quality_permille);
            metal_access_weighted += u128::from(cell.metals.remaining_quantity)
                * u128::from(cell.metals.accessibility_permille);
            construction_quality_weighted += u128::from(cell.construction.remaining_quantity)
                * u128::from(cell.construction.quality_permille);
            construction_access_weighted += u128::from(cell.construction.remaining_quantity)
                * u128::from(cell.construction.accessibility_permille);
        }

        aggregate.energy_quality_permille =
            weighted_permille(energy_quality_weighted, aggregate.energy_remaining);
        aggregate.energy_accessibility_permille =
            weighted_permille(energy_access_weighted, aggregate.energy_remaining);
        aggregate.metals_quality_permille =
            weighted_permille(metal_quality_weighted, aggregate.metals_remaining);
        aggregate.metals_accessibility_permille =
            weighted_permille(metal_access_weighted, aggregate.metals_remaining);
        aggregate.construction_quality_permille = weighted_permille(
            construction_quality_weighted,
            aggregate.construction_remaining,
        );
        aggregate.construction_accessibility_permille = weighted_permille(
            construction_access_weighted,
            aggregate.construction_remaining,
        );
        Some(aggregate)
    }

    /// Confirms resource cells line up with the terrain and that ocean cells do
    /// not accidentally contain land-resource endowments.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError`] on alignment or ocean-resource violations.
    pub fn validate_against_terrain(&self, terrain: &TerrainState) -> Result<(), ResourceError> {
        if self.cells.len() != terrain.samples.len() {
            return Err(ResourceError::WrongCellCount {
                found: self.cells.len(),
            });
        }
        for (index, (terrain_sample, resource_cell)) in
            terrain.samples.iter().zip(&self.cells).enumerate()
        {
            resource_cell
                .validate()
                .map_err(|source| ResourceError::InvalidCell {
                    index,
                    source: source.kind(),
                })?;
            if terrain_sample.elevation_m < terrain.sea_level_m
                && *resource_cell != ResourceCellState::OCEAN
            {
                return Err(ResourceError::OceanContainsLandResources { index });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ResourceAggregate {
    pub food_capacity_tonnes_per_year: u64,
    pub energy_remaining: u64,
    pub energy_quality_permille: u16,
    pub energy_accessibility_permille: u16,
    pub metals_remaining: u64,
    pub metals_quality_permille: u16,
    pub metals_accessibility_permille: u16,
    pub construction_remaining: u64,
    pub construction_quality_permille: u16,
    pub construction_accessibility_permille: u16,
}

/// Converts Stage 2.3 agricultural suitability to a renewable annual food base
/// for one 100 km² terrain sample. The 50,000-tonne neutral reference is a TEST
/// calibration constant, not an empirical yield claim.
#[must_use]
pub fn food_capacity_from_terrain_effects(
    effects: &TerrainEffectField,
    index: usize,
) -> Option<u32> {
    let terrain_effect = effects.sample(index)?;
    let capacity =
        50_000_u64.saturating_mul(u64::from(terrain_effect.agriculture_yield_permille)) / 1_000;
    u32::try_from(capacity).ok()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceErrorKind {
    InvalidQuality,
    InvalidAccessibility,
    RemainingExceedsInitial,
    InvalidEmptyDeposit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceError {
    InvalidQuality(u16),
    InvalidAccessibility(u16),
    RemainingExceedsInitial,
    InvalidEmptyDeposit,
    WrongCellCount {
        found: usize,
    },
    InvalidCell {
        index: usize,
        source: ResourceErrorKind,
    },
    OceanContainsLandResources {
        index: usize,
    },
}

impl ResourceError {
    const fn kind(self) -> ResourceErrorKind {
        match self {
            Self::InvalidQuality(_) => ResourceErrorKind::InvalidQuality,
            Self::InvalidAccessibility(_) => ResourceErrorKind::InvalidAccessibility,
            Self::RemainingExceedsInitial => ResourceErrorKind::RemainingExceedsInitial,
            Self::InvalidEmptyDeposit
            | Self::WrongCellCount { .. }
            | Self::InvalidCell { .. }
            | Self::OceanContainsLandResources { .. } => ResourceErrorKind::InvalidEmptyDeposit,
        }
    }
}

impl core::fmt::Display for ResourceError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidQuality(value) => {
                write!(formatter, "resource quality {value} exceeds 1000")
            }
            Self::InvalidAccessibility(value) => {
                write!(formatter, "resource accessibility {value} exceeds 1000")
            }
            Self::RemainingExceedsInitial => {
                formatter.write_str("remaining resource quantity exceeds initial quantity")
            }
            Self::InvalidEmptyDeposit => {
                formatter.write_str("zero-quantity deposit contains nonzero metadata")
            }
            Self::WrongCellCount { found } => write!(
                formatter,
                "resource field requires {TERRAIN_SAMPLE_COUNT} cells, found {found}"
            ),
            Self::InvalidCell { index, source } => {
                write!(formatter, "resource cell {index} is invalid: {source:?}")
            }
            Self::OceanContainsLandResources { index } => {
                write!(
                    formatter,
                    "ocean terrain sample {index} contains land resources"
                )
            }
        }
    }
}

impl std::error::Error for ResourceError {}

fn weighted_permille(weighted_total: u128, quantity: u64) -> u16 {
    if quantity == 0 {
        return 0;
    }
    let value = weighted_total / u128::from(quantity);
    u16::try_from(value).unwrap_or(1_000)
}

#[cfg(test)]
mod tests {
    use super::{ResourceDeposit, ResourceError};

    #[test]
    fn extraction_cannot_exceed_remaining_stock() {
        let mut deposit = ResourceDeposit::new(100, 700, 600).expect("deposit valid");
        let first = deposit.extract(40);
        assert_eq!(first.extracted_quantity, 40);
        assert_eq!(deposit.remaining_quantity, 60);
        let second = deposit.extract(100);
        assert_eq!(second.extracted_quantity, 60);
        assert_eq!(deposit.remaining_quantity, 0);
        assert_eq!(deposit.depletion_permille(), 1_000);
    }

    #[test]
    fn invalid_quality_is_rejected() {
        assert!(matches!(
            ResourceDeposit::new(100, 1_001, 500),
            Err(ResourceError::InvalidQuality(1_001))
        ));
    }
}
