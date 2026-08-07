#![forbid(unsafe_code)]

use simulation_model::{
    BiomeClass, ReliefClass, TerrainError, TerrainSample, TerrainState, TERRAIN_GRID_SIDE,
    TERRAIN_SAMPLE_COUNT,
};

const CONTINENTAL_SALT_1: u64 = 0x434f_4e54_0000_0001;
const CONTINENTAL_SALT_2: u64 = 0x434f_4e54_0000_0002;
const CONTINENTAL_SALT_3: u64 = 0x434f_4e54_0000_0003;
const CONTINENTAL_SALT_4: u64 = 0x434f_4e54_0000_0004;
const RIDGE_SALT: u64 = 0x5249_4447_4500_0001;
const MOISTURE_SALT_1: u64 = 0x4d4f_4953_5400_0001;
const MOISTURE_SALT_2: u64 = 0x4d4f_4953_5400_0002;
const Q16_MAX: i64 = 65_535;

/// Generates the canonical TEST terrain from a world seed.
///
/// The generator uses only integer arithmetic and stable hash/value-noise
/// operations. It therefore produces byte-identical authoritative terrain for
/// the same seed independent of renderer, wall-clock timing or platform float
/// behavior.
///
/// # Errors
///
/// Returns [`TerrainError`] only if the generated terrain violates the canonical
/// Stage 2.2 terrain representation.
pub fn generate_trial_terrain(seed: u64) -> Result<TerrainState, TerrainError> {
    let side = usize::from(TERRAIN_GRID_SIDE);
    let mut raw = Vec::with_capacity(TERRAIN_SAMPLE_COUNT);
    let mut ridges = Vec::with_capacity(TERRAIN_SAMPLE_COUNT);
    let mut moisture = Vec::with_capacity(TERRAIN_SAMPLE_COUNT);

    for z in 0..TERRAIN_GRID_SIDE {
        for x in 0..TERRAIN_GRID_SIDE {
            raw.push(raw_continental_signal(seed, x, z));
            ridges.push(ridge_signal(seed, x, z));
            moisture.push(moisture_permille(seed, x, z));
        }
    }

    let mut ordered = raw.clone();
    ordered.sort_unstable();
    let sea_index = ordered.len().saturating_mul(55) / 100;
    let sea_threshold = ordered
        .get(sea_index.min(ordered.len().saturating_sub(1)))
        .copied()
        .unwrap_or(0);

    let mut elevations = Vec::with_capacity(TERRAIN_SAMPLE_COUNT);
    for (index, (&signal, &ridge)) in raw.iter().zip(&ridges).enumerate() {
        let delta = signal - sea_threshold;
        let mut elevation = i64::from(delta) * 4_500 / 20_000;
        if elevation >= 0 {
            let continental_strength = i64::from(delta.clamp(0, 18_000));
            let ridge_bonus = i64::from(ridge) * 2_200 / 32_768;
            elevation += ridge_bonus * continental_strength / 18_000;
        }
        elevation = elevation.clamp(-6_000, 6_500);

        let x = index % side;
        let z = index / side;
        let edge_distance = x.min(z).min(side - 1 - x).min(side - 1 - z);
        if edge_distance < 3 {
            let edge = i64::try_from(edge_distance).unwrap_or(0);
            let forced_ocean = -1_000 - (3 - edge) * 500;
            elevation = elevation.min(forced_ocean);
        }

        elevations.push(i16::try_from(elevation).unwrap_or(if elevation < 0 {
            i16::MIN
        } else {
            i16::MAX
        }));
    }

    let mut samples = Vec::with_capacity(TERRAIN_SAMPLE_COUNT);
    for index in 0..TERRAIN_SAMPLE_COUNT {
        let elevation = elevations[index];
        let x = index % side;
        let z = index / side;
        let relief = classify_relief(&elevations, x, z, elevation);
        let biome = classify_biome(elevation, moisture[index]);
        samples.push(TerrainSample {
            elevation_m: elevation,
            moisture_permille: moisture[index],
            relief,
            biome,
        });
    }

    TerrainState::new_trial(samples)
}

fn raw_continental_signal(seed: u64, x: u16, z: u16) -> i32 {
    let low = value_noise(seed, x, z, 64, CONTINENTAL_SALT_1);
    let medium = value_noise(seed, x, z, 32, CONTINENTAL_SALT_2);
    let high = value_noise(seed, x, z, 16, CONTINENTAL_SALT_3);
    let detail = value_noise(seed, x, z, 8, CONTINENTAL_SALT_4);
    let fractal = (low * 8 + medium * 4 + high * 2 + detail) / 15;

    let center = i32::from(TERRAIN_GRID_SIDE - 1) / 2;
    let dx = (i32::from(x) - center).unsigned_abs();
    let dz = (i32::from(z) - center).unsigned_abs();
    let center_u32 = u32::try_from(center).unwrap_or(1);
    let normalized_x = i64::from(dx) * Q16_MAX / i64::from(center_u32);
    let normalized_z = i64::from(dz) * Q16_MAX / i64::from(center_u32);
    let edge = normalized_x.max(normalized_z);
    let edge_squared = edge * edge / Q16_MAX;
    let edge_cubed = edge_squared * edge / Q16_MAX;
    let edge_penalty = edge_cubed * 18_000 / Q16_MAX;

    fractal - i32::try_from(edge_penalty).unwrap_or(i32::MAX)
}

fn ridge_signal(seed: u64, x: u16, z: u16) -> i32 {
    32_768 - value_noise(seed, x, z, 16, RIDGE_SALT).abs()
}

fn moisture_permille(seed: u64, x: u16, z: u16) -> u16 {
    let broad = value_noise(seed, x, z, 48, MOISTURE_SALT_1);
    let detail = value_noise(seed, x, z, 16, MOISTURE_SALT_2);
    let combined = (broad * 2 + detail) / 3;
    let shifted = i64::from(combined) + 32_768;
    let permille = (shifted * 1_000 / 65_535).clamp(0, 1_000);
    u16::try_from(permille).unwrap_or(0)
}

fn value_noise(seed: u64, x: u16, z: u16, scale: u16, salt: u64) -> i32 {
    let x0 = u32::from(x / scale);
    let z0 = u32::from(z / scale);
    let x_fraction = u32::from(x % scale) * 65_535 / u32::from(scale);
    let z_fraction = u32::from(z % scale) * 65_535 / u32::from(scale);
    let smooth_x = smooth_q16(x_fraction);
    let smooth_z = smooth_q16(z_fraction);

    let north = lerp_i32(
        lattice_value(seed, x0, z0, salt),
        lattice_value(seed, x0 + 1, z0, salt),
        smooth_x,
    );
    let south = lerp_i32(
        lattice_value(seed, x0, z0 + 1, salt),
        lattice_value(seed, x0 + 1, z0 + 1, salt),
        smooth_x,
    );
    lerp_i32(north, south, smooth_z)
}

fn smooth_q16(value: u32) -> u32 {
    let value = u64::from(value);
    let maximum = 65_535_u64;
    let smoothed = value * value * (3 * maximum - 2 * value) / (maximum * maximum);
    u32::try_from(smoothed).unwrap_or(65_535)
}

fn lerp_i32(start: i32, end: i32, factor_q16: u32) -> i32 {
    let delta = i64::from(end) - i64::from(start);
    let interpolated = i64::from(start) + delta * i64::from(factor_q16) / Q16_MAX;
    i32::try_from(interpolated).unwrap_or(if interpolated < 0 { i32::MIN } else { i32::MAX })
}

fn lattice_value(seed: u64, x: u32, z: u32, salt: u64) -> i32 {
    let value = seed
        ^ u64::from(x).wrapping_mul(0x9e37_79b9_7f4a_7c15)
        ^ u64::from(z).wrapping_mul(0xd6e8_feb8_6659_fd93)
        ^ salt;
    let mixed = mix64(value);
    let upper = u16::try_from(mixed >> 48).unwrap_or(0);
    i32::from(upper) - 32_768
}

fn mix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

fn classify_relief(elevations: &[i16], x: usize, z: usize, elevation: i16) -> ReliefClass {
    if elevation < -1_200 {
        return ReliefClass::DeepOcean;
    }
    if elevation < 0 {
        return ReliefClass::ShallowOcean;
    }
    if has_adjacent_ocean(elevations, x, z) {
        return ReliefClass::Coast;
    }
    if elevation < 450 {
        ReliefClass::Plains
    } else if elevation < 1_500 {
        ReliefClass::Hills
    } else {
        ReliefClass::Mountains
    }
}

fn has_adjacent_ocean(elevations: &[i16], x: usize, z: usize) -> bool {
    let side = usize::from(TERRAIN_GRID_SIDE);
    let min_x = x.saturating_sub(1);
    let min_z = z.saturating_sub(1);
    let max_x = (x + 1).min(side - 1);
    let max_z = (z + 1).min(side - 1);

    for neighbor_z in min_z..=max_z {
        for neighbor_x in min_x..=max_x {
            if neighbor_x == x && neighbor_z == z {
                continue;
            }
            if elevations[neighbor_z * side + neighbor_x] < 0 {
                return true;
            }
        }
    }
    false
}

fn classify_biome(elevation: i16, moisture_permille: u16) -> BiomeClass {
    if elevation < 0 {
        return BiomeClass::Ocean;
    }
    if elevation > 2_200 {
        return BiomeClass::Alpine;
    }
    if elevation < 250 && moisture_permille > 780 {
        return BiomeClass::Wetland;
    }
    if moisture_permille < 260 {
        return BiomeClass::Desert;
    }
    if moisture_permille > 640 {
        return BiomeClass::Forest;
    }
    BiomeClass::Grassland
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use simulation_model::{BiomeClass, ReliefClass, TERRAIN_GRID_SIDE, TERRAIN_SAMPLE_COUNT};

    use super::generate_trial_terrain;

    #[test]
    fn same_seed_generates_identical_terrain() {
        let first = generate_trial_terrain(2026).expect("terrain should generate");
        let second = generate_trial_terrain(2026).expect("terrain should generate");
        assert_eq!(first, second);
    }

    #[test]
    fn different_seed_changes_terrain() {
        let first = generate_trial_terrain(2026).expect("terrain should generate");
        let second = generate_trial_terrain(2027).expect("terrain should generate");
        assert_ne!(first.samples, second.samples);
    }

    #[test]
    fn canonical_seed_has_balanced_land_and_continuous_outer_ocean() {
        let terrain = generate_trial_terrain(2026).expect("terrain should generate");
        let land = terrain.land_sample_count();
        assert!(land * 100 >= TERRAIN_SAMPLE_COUNT * 40);
        assert!(land * 100 <= TERRAIN_SAMPLE_COUNT * 50);

        let side = usize::from(TERRAIN_GRID_SIDE);
        for index in 0..side {
            assert!(terrain.samples[index].elevation_m < 0);
            assert!(terrain.samples[(side - 1) * side + index].elevation_m < 0);
            assert!(terrain.samples[index * side].elevation_m < 0);
            assert!(terrain.samples[index * side + side - 1].elevation_m < 0);
        }
    }

    #[test]
    fn canonical_seed_exposes_required_relief_and_biome_variety() {
        let terrain = generate_trial_terrain(2026).expect("terrain should generate");
        let relief: BTreeSet<_> = terrain
            .samples
            .iter()
            .map(|sample| sample.relief as u8)
            .collect();
        let biome: BTreeSet<_> = terrain
            .samples
            .iter()
            .map(|sample| sample.biome as u8)
            .collect();

        for required in [
            ReliefClass::DeepOcean,
            ReliefClass::ShallowOcean,
            ReliefClass::Coast,
            ReliefClass::Plains,
            ReliefClass::Hills,
            ReliefClass::Mountains,
        ] {
            assert!(relief.contains(&(required as u8)));
        }
        for required in [
            BiomeClass::Ocean,
            BiomeClass::Grassland,
            BiomeClass::Forest,
            BiomeClass::Desert,
            BiomeClass::Wetland,
            BiomeClass::Alpine,
        ] {
            assert!(biome.contains(&(required as u8)));
        }
    }

    #[test]
    fn canonical_seed_limits_neighbor_elevation_discontinuity() {
        let terrain = generate_trial_terrain(2026).expect("terrain should generate");
        let side = usize::from(TERRAIN_GRID_SIDE);
        let mut maximum_delta = 0_i32;
        for z in 0..side {
            for x in 0..side {
                let current = i32::from(terrain.samples[z * side + x].elevation_m);
                if x + 1 < side {
                    let east = i32::from(terrain.samples[z * side + x + 1].elevation_m);
                    maximum_delta = maximum_delta.max((current - east).abs());
                }
                if z + 1 < side {
                    let south = i32::from(terrain.samples[(z + 1) * side + x].elevation_m);
                    maximum_delta = maximum_delta.max((current - south).abs());
                }
            }
        }
        assert!(maximum_delta <= 3_500);
    }
}
