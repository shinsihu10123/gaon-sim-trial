pub(super) fn ensure_seeded_island(elevations: &mut [i16], side: usize, seed: u64) -> bool {
    if side < 11 || elevations.len() != side * side {
        return false;
    }

    let total = side * side;
    let total_u64 = u64::try_from(total).unwrap_or(1);
    let start_u64 = mix64(seed ^ 0x4953_4c41_4e44_0001) % total_u64;
    let start = usize::try_from(start_u64).unwrap_or(0);

    for offset in 0..total {
        let index = (start + offset) % total;
        let x = index % side;
        let z = index / side;
        if x < 5 || z < 5 || x + 5 >= side || z + 5 >= side {
            continue;
        }
        if !square_is_ocean(elevations, side, x, z, 3) {
            continue;
        }

        for delta_z in -2_i32..=2 {
            for delta_x in -2_i32..=2 {
                let target_x = usize::try_from(i32::try_from(x).unwrap_or(0) + delta_x).unwrap_or(x);
                let target_z = usize::try_from(i32::try_from(z).unwrap_or(0) + delta_z).unwrap_or(z);
                let target = target_z * side + target_x;
                elevations[target] = elevations[target].min(-120);
            }
        }

        for delta_z in -1_i32..=1 {
            for delta_x in -1_i32..=1 {
                let target_x = usize::try_from(i32::try_from(x).unwrap_or(0) + delta_x).unwrap_or(x);
                let target_z = usize::try_from(i32::try_from(z).unwrap_or(0) + delta_z).unwrap_or(z);
                let target = target_z * side + target_x;
                elevations[target] = if delta_x == 0 && delta_z == 0 {
                    320
                } else if delta_x == 0 || delta_z == 0 {
                    190
                } else {
                    110
                };
            }
        }
        return true;
    }

    false
}

fn square_is_ocean(
    elevations: &[i16],
    side: usize,
    center_x: usize,
    center_z: usize,
    radius: usize,
) -> bool {
    let min_x = center_x.saturating_sub(radius);
    let min_z = center_z.saturating_sub(radius);
    let max_x = (center_x + radius).min(side - 1);
    let max_z = (center_z + radius).min(side - 1);
    (min_z..=max_z).all(|z| (min_x..=max_x).all(|x| elevations[z * side + x] < 0))
}

fn mix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

#[cfg(test)]
mod tests {
    use simulation_model::TerrainState;

    use super::ensure_seeded_island;

    #[test]
    fn seeded_island_creates_secondary_landmass() {
        let side = 25;
        let mut elevations = vec![-500_i16; side * side];
        for z in 10..=14 {
            for x in 10..=14 {
                elevations[z * side + x] = 200;
            }
        }
        assert!(ensure_seeded_island(&mut elevations, side, 7));

        let _type_anchor: Option<TerrainState> = None;
        let land_count = elevations.iter().filter(|&&elevation| elevation >= 0).count();
        assert!(land_count > 25);
    }
}
