use crate::{
    BiomeClass, ReliefClass, TerrainHydrology, TerrainSample, TerrainState, NO_DOWNSTREAM_INDEX,
};

/// Terrain-only multipliers and capacities consumed by later economic,
/// infrastructure and military systems.
///
/// Multipliers use permille fixed point: `1000` means neutral, `1250` means
/// 1.25× and `800` means 0.80×. Carrying capacity is expressed directly as
/// people per square kilometre before urban infrastructure or technology.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerrainEffects {
    pub agriculture_yield_permille: u16,
    pub construction_cost_permille: u16,
    pub movement_cost_permille: u16,
    pub defense_multiplier_permille: u16,
    pub port_feasibility_permille: u16,
    pub carrying_capacity_people_per_km2: u16,
    pub productivity_multiplier_permille: u16,
}

impl TerrainEffects {
    pub const OCEAN: Self = Self {
        agriculture_yield_permille: 0,
        construction_cost_permille: 0,
        movement_cost_permille: 0,
        defense_multiplier_permille: 0,
        port_feasibility_permille: 0,
        carrying_capacity_people_per_km2: 0,
        productivity_multiplier_permille: 0,
    };
}

/// Complete deterministic Stage 2.3 effect field aligned one-to-one with the
/// canonical terrain samples.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerrainEffectField {
    pub samples: Vec<TerrainEffects>,
}

impl TerrainEffectField {
    #[must_use]
    pub fn sample(&self, index: usize) -> Option<&TerrainEffects> {
        self.samples.get(index)
    }

    /// Produces an unweighted mean suitable for a later region whose terrain
    /// membership is represented by canonical sample indices.
    #[must_use]
    pub fn aggregate_indices(&self, indices: &[usize]) -> Option<TerrainEffects> {
        if indices.is_empty() {
            return None;
        }
        let mut agriculture = 0_u64;
        let mut construction = 0_u64;
        let mut movement = 0_u64;
        let mut defense = 0_u64;
        let mut port = 0_u64;
        let mut carrying = 0_u64;
        let mut productivity = 0_u64;

        for &index in indices {
            let value = self.samples.get(index)?;
            agriculture += u64::from(value.agriculture_yield_permille);
            construction += u64::from(value.construction_cost_permille);
            movement += u64::from(value.movement_cost_permille);
            defense += u64::from(value.defense_multiplier_permille);
            port += u64::from(value.port_feasibility_permille);
            carrying += u64::from(value.carrying_capacity_people_per_km2);
            productivity += u64::from(value.productivity_multiplier_permille);
        }

        let count = u64::try_from(indices.len()).ok()?;
        Some(TerrainEffects {
            agriculture_yield_permille: average_u16(agriculture, count),
            construction_cost_permille: average_u16(construction, count),
            movement_cost_permille: average_u16(movement, count),
            defense_multiplier_permille: average_u16(defense, count),
            port_feasibility_permille: average_u16(port, count),
            carrying_capacity_people_per_km2: average_u16(carrying, count),
            productivity_multiplier_permille: average_u16(productivity, count),
        })
    }
}

/// Derives all Stage 2.3 effects from canonical terrain plus deterministic
/// hydrology. No renderer, country state or infrastructure state participates.
#[must_use]
pub fn derive_terrain_effects(terrain: &TerrainState) -> TerrainEffectField {
    let hydrology = terrain.derive_hydrology();
    let samples = terrain
        .samples
        .iter()
        .enumerate()
        .map(|(index, sample)| effects_for_sample(terrain, &hydrology, index, *sample))
        .collect();
    TerrainEffectField { samples }
}

fn effects_for_sample(
    terrain: &TerrainState,
    hydrology: &TerrainHydrology,
    index: usize,
    sample: TerrainSample,
) -> TerrainEffects {
    if sample.elevation_m < terrain.sea_level_m {
        return TerrainEffects::OCEAN;
    }

    let river_order = hydrology.river_orders[index];
    let river_mouth = is_river_mouth(terrain, hydrology, index);

    let agriculture = agriculture_yield(sample, river_order);
    let construction = construction_cost(sample);
    let movement = movement_cost(sample, river_order);
    let defense = defense_multiplier(sample);
    let port = port_feasibility(sample, river_mouth);
    let carrying = carrying_capacity(sample, agriculture, river_order);
    let productivity = productivity_multiplier(sample, construction, movement, river_order);

    TerrainEffects {
        agriculture_yield_permille: agriculture,
        construction_cost_permille: construction,
        movement_cost_permille: movement,
        defense_multiplier_permille: defense,
        port_feasibility_permille: port,
        carrying_capacity_people_per_km2: carrying,
        productivity_multiplier_permille: productivity,
    }
}

fn agriculture_yield(sample: TerrainSample, river_order: u8) -> u16 {
    let biome_base = match sample.biome {
        BiomeClass::Ocean => 0,
        BiomeClass::Grassland => 900,
        BiomeClass::Forest => 650,
        BiomeClass::Desert => 180,
        BiomeClass::Wetland => 720,
        BiomeClass::Alpine => 100,
    };
    let relief_adjustment = match sample.relief {
        ReliefClass::DeepOcean | ReliefClass::ShallowOcean => -1_500,
        ReliefClass::Coast => 60,
        ReliefClass::Plains => 160,
        ReliefClass::Hills => -90,
        ReliefClass::Mountains => -420,
    };
    let moisture_distance = i32::from(sample.moisture_permille).abs_diff(550);
    let moisture_bonus = 300_i32 - i32::try_from(moisture_distance / 2).unwrap_or(300);
    let river_bonus = i32::from(river_order) * 55;
    clamp_u16(
        biome_base + relief_adjustment + moisture_bonus.max(0) + river_bonus,
        0,
        1_400,
    )
}

fn construction_cost(sample: TerrainSample) -> u16 {
    let relief = match sample.relief {
        ReliefClass::DeepOcean | ReliefClass::ShallowOcean => 0,
        ReliefClass::Coast => 1_100,
        ReliefClass::Plains => 1_000,
        ReliefClass::Hills => 1_250,
        ReliefClass::Mountains => 1_750,
    };
    let cover = match sample.biome {
        BiomeClass::Ocean | BiomeClass::Grassland => 0,
        BiomeClass::Forest => 150,
        BiomeClass::Desert => 50,
        BiomeClass::Wetland => 350,
        BiomeClass::Alpine => 250,
    };
    clamp_u16(relief + cover, 1_000, 2_200)
}

fn movement_cost(sample: TerrainSample, river_order: u8) -> u16 {
    let relief = match sample.relief {
        ReliefClass::DeepOcean | ReliefClass::ShallowOcean => 0,
        ReliefClass::Coast => 1_050,
        ReliefClass::Plains => 1_000,
        ReliefClass::Hills => 1_250,
        ReliefClass::Mountains => 1_700,
    };
    let cover = match sample.biome {
        BiomeClass::Ocean | BiomeClass::Grassland => 0,
        BiomeClass::Forest => 150,
        BiomeClass::Desert => 100,
        BiomeClass::Wetland => 300,
        BiomeClass::Alpine => 200,
    };
    let river_crossing = i32::from(river_order) * 45;
    clamp_u16(relief + cover + river_crossing, 1_000, 2_200)
}

fn defense_multiplier(sample: TerrainSample) -> u16 {
    let relief = match sample.relief {
        ReliefClass::DeepOcean | ReliefClass::ShallowOcean => 0,
        ReliefClass::Coast => 1_000,
        ReliefClass::Plains => 900,
        ReliefClass::Hills => 1_200,
        ReliefClass::Mountains => 1_500,
    };
    let cover = match sample.biome {
        BiomeClass::Forest => 120,
        BiomeClass::Wetland => 80,
        BiomeClass::Alpine => 100,
        _ => 0,
    };
    clamp_u16(relief + cover, 850, 1_700)
}

fn port_feasibility(sample: TerrainSample, river_mouth: bool) -> u16 {
    if sample.relief != ReliefClass::Coast {
        return 0;
    }
    let low_elevation_bonus = if sample.elevation_m <= 250 { 150 } else { 0 };
    let river_mouth_bonus = if river_mouth { 150 } else { 0 };
    let wetland_penalty = if sample.biome == BiomeClass::Wetland {
        100
    } else {
        0
    };
    clamp_u16(
        700 + low_elevation_bonus + river_mouth_bonus - wetland_penalty,
        0,
        1_000,
    )
}

fn carrying_capacity(sample: TerrainSample, agriculture: u16, river_order: u8) -> u16 {
    let coast_bonus = if sample.relief == ReliefClass::Coast {
        25
    } else {
        0
    };
    let river_bonus = i32::from(river_order) * 10;
    let altitude_penalty = if sample.elevation_m > 2_000 { 40 } else { 0 };
    let raw = 10_i32 + i32::from(agriculture) / 6 + coast_bonus + river_bonus - altitude_penalty;
    clamp_u16(raw, 5, 280)
}

fn productivity_multiplier(
    sample: TerrainSample,
    construction: u16,
    movement: u16,
    river_order: u8,
) -> u16 {
    let construction_penalty = i32::from(construction.saturating_sub(1_000)) / 5;
    let movement_penalty = i32::from(movement.saturating_sub(1_000)) / 3;
    let coast_bonus = if sample.relief == ReliefClass::Coast {
        50
    } else {
        0
    };
    let river_bonus = i32::from(river_order) * 30;
    clamp_u16(
        1_000 - construction_penalty - movement_penalty + coast_bonus + river_bonus,
        600,
        1_150,
    )
}

fn is_river_mouth(terrain: &TerrainState, hydrology: &TerrainHydrology, index: usize) -> bool {
    if hydrology.river_orders[index] == 0 {
        return false;
    }
    let downstream = hydrology.downstream_indices[index];
    if downstream == NO_DOWNSTREAM_INDEX {
        return false;
    }
    let Ok(downstream_index) = usize::try_from(downstream) else {
        return false;
    };
    terrain
        .samples
        .get(downstream_index)
        .is_some_and(|sample| sample.elevation_m < terrain.sea_level_m)
}

fn clamp_u16(value: i32, minimum: i32, maximum: i32) -> u16 {
    u16::try_from(value.clamp(minimum, maximum)).unwrap_or(u16::MAX)
}

fn average_u16(total: u64, count: u64) -> u16 {
    let rounded = (total + count / 2) / count;
    u16::try_from(rounded).unwrap_or(u16::MAX)
}

#[cfg(test)]
mod tests {
    use crate::{BiomeClass, ReliefClass, TerrainSample};

    use super::{agriculture_yield, construction_cost, defense_multiplier, movement_cost};

    #[test]
    fn plains_are_cheaper_to_build_and_move_across_than_mountains() {
        let plains = TerrainSample {
            elevation_m: 200,
            moisture_permille: 550,
            relief: ReliefClass::Plains,
            biome: BiomeClass::Grassland,
        };
        let mountains = TerrainSample {
            elevation_m: 2_000,
            moisture_permille: 550,
            relief: ReliefClass::Mountains,
            biome: BiomeClass::Alpine,
        };
        assert!(construction_cost(plains) < construction_cost(mountains));
        assert!(movement_cost(plains, 0) < movement_cost(mountains, 0));
        assert!(defense_multiplier(plains) < defense_multiplier(mountains));
    }

    #[test]
    fn temperate_grassland_outproduces_dry_desert() {
        let grassland = TerrainSample {
            elevation_m: 100,
            moisture_permille: 550,
            relief: ReliefClass::Plains,
            biome: BiomeClass::Grassland,
        };
        let desert = TerrainSample {
            elevation_m: 100,
            moisture_permille: 100,
            relief: ReliefClass::Plains,
            biome: BiomeClass::Desert,
        };
        assert!(agriculture_yield(grassland, 0) > agriculture_yield(desert, 0));
    }
}
