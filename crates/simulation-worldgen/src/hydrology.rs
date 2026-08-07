use std::collections::{BTreeMap, BTreeSet, VecDeque};

use simulation_model::{
    TerrainHydrology, TerrainLandmasses, NO_DOWNSTREAM_INDEX, TERRAIN_SAMPLE_COUNT,
};

const RIVER_HEADWATER_THRESHOLD: u32 = 8;
const RIVER_MEDIUM_THRESHOLD: u32 = 32;
const RIVER_MAJOR_THRESHOLD: u32 = 128;

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

pub(super) fn derive_landmasses(elevations: &[i16], side: usize) -> TerrainLandmasses {
    let mut landmass_ids = vec![0_u16; elevations.len()];
    let mut component_sizes = Vec::new();
    let mut next_id = 1_u16;

    for start in 0..elevations.len() {
        if elevations[start] < 0 || landmass_ids[start] != 0 {
            continue;
        }

        let id = next_id;
        next_id = next_id.saturating_add(1);
        let mut size = 0_usize;
        let mut queue = VecDeque::from([start]);
        landmass_ids[start] = id;

        while let Some(index) = queue.pop_front() {
            size += 1;
            let x = index % side;
            let z = index / side;
            for neighbor in neighbor_indices(x, z, side) {
                if elevations[neighbor] >= 0 && landmass_ids[neighbor] == 0 {
                    landmass_ids[neighbor] = id;
                    queue.push_back(neighbor);
                }
            }
        }
        component_sizes.push((id, size));
    }

    let mainland_id = component_sizes
        .iter()
        .max_by(|(left_id, left_size), (right_id, right_size)| {
            left_size
                .cmp(right_size)
                .then_with(|| right_id.cmp(left_id))
        })
        .map(|(id, _)| *id)
        .unwrap_or(0);
    let island_landmass_ids = component_sizes
        .iter()
        .filter_map(|(id, _)| (*id != mainland_id).then_some(*id))
        .collect();

    TerrainLandmasses {
        landmass_ids,
        island_landmass_ids,
    }
}

pub(super) fn derive_hydrology(elevations: &[i16], side: usize) -> TerrainHydrology {
    let mut downstream_indices = vec![NO_DOWNSTREAM_INDEX; elevations.len()];

    for index in 0..elevations.len() {
        if elevations[index] < 0 {
            continue;
        }
        let x = index % side;
        let z = index / side;
        let current = elevations[index];
        let mut best: Option<(i16, usize)> = None;
        for neighbor in neighbor_indices(x, z, side) {
            let candidate = elevations[neighbor];
            if candidate >= current {
                continue;
            }
            if best.is_none_or(|(best_elevation, best_index)| {
                candidate < best_elevation || (candidate == best_elevation && neighbor < best_index)
            }) {
                best = Some((candidate, neighbor));
            }
        }
        if let Some((_, downstream)) = best {
            downstream_indices[index] = u32::try_from(downstream).unwrap_or(NO_DOWNSTREAM_INDEX);
        }
    }

    let mut flow_accumulation = elevations
        .iter()
        .map(|&elevation| u32::from(elevation >= 0))
        .collect::<Vec<_>>();
    let mut descending = (0..elevations.len())
        .filter(|&index| elevations[index] >= 0)
        .collect::<Vec<_>>();
    descending.sort_by(|left, right| {
        elevations[*right]
            .cmp(&elevations[*left])
            .then_with(|| left.cmp(right))
    });

    for index in descending {
        let downstream = downstream_indices[index];
        if downstream == NO_DOWNSTREAM_INDEX {
            continue;
        }
        let Ok(downstream_index) = usize::try_from(downstream) else {
            continue;
        };
        if elevations.get(downstream_index).is_some_and(|&elevation| elevation >= 0) {
            flow_accumulation[downstream_index] = flow_accumulation[downstream_index]
                .saturating_add(flow_accumulation[index]);
        }
    }

    let terminal_keys = (0..elevations.len())
        .filter(|&index| elevations[index] >= 0)
        .map(|index| terminal_key(index, elevations, &downstream_indices))
        .collect::<BTreeSet<_>>();
    let terminal_to_basin = terminal_keys
        .into_iter()
        .enumerate()
        .map(|(offset, terminal)| {
            let id = u16::try_from(offset + 1).unwrap_or(u16::MAX);
            (terminal, id)
        })
        .collect::<BTreeMap<_, _>>();

    let mut drainage_basin_ids = vec![0_u16; elevations.len()];
    let mut river_orders = vec![0_u8; elevations.len()];
    for index in 0..elevations.len() {
        if elevations[index] < 0 {
            continue;
        }
        let terminal = terminal_key(index, elevations, &downstream_indices);
        drainage_basin_ids[index] = terminal_to_basin.get(&terminal).copied().unwrap_or(1);
        if downstream_indices[index] != NO_DOWNSTREAM_INDEX {
            river_orders[index] = match flow_accumulation[index] {
                value if value >= RIVER_MAJOR_THRESHOLD => 3,
                value if value >= RIVER_MEDIUM_THRESHOLD => 2,
                value if value >= RIVER_HEADWATER_THRESHOLD => 1,
                _ => 0,
            };
        }
    }

    TerrainHydrology {
        downstream_indices,
        drainage_basin_ids,
        flow_accumulation,
        river_orders,
    }
}

fn terminal_key(index: usize, elevations: &[i16], downstream_indices: &[u32]) -> usize {
    let mut current = index;
    loop {
        let downstream = downstream_indices[current];
        if downstream == NO_DOWNSTREAM_INDEX {
            return elevations.len() + current;
        }
        let Ok(next) = usize::try_from(downstream) else {
            return elevations.len() + current;
        };
        if elevations.get(next).is_none_or(|&elevation| elevation < 0) {
            return next;
        }
        current = next;
    }
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

fn neighbor_indices(x: usize, z: usize, side: usize) -> Vec<usize> {
    let min_x = x.saturating_sub(1);
    let min_z = z.saturating_sub(1);
    let max_x = (x + 1).min(side - 1);
    let max_z = (z + 1).min(side - 1);
    let mut neighbors = Vec::with_capacity(8);
    for neighbor_z in min_z..=max_z {
        for neighbor_x in min_x..=max_x {
            if neighbor_x != x || neighbor_z != z {
                neighbors.push(neighbor_z * side + neighbor_x);
            }
        }
    }
    neighbors
}

fn mix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

#[cfg(test)]
mod tests {
    use super::{derive_hydrology, derive_landmasses, ensure_seeded_island};

    #[test]
    fn seeded_island_creates_secondary_landmass() {
        let side = 17;
        let mut elevations = vec![-500_i16; side * side];
        for z in 6..=10 {
            for x in 6..=10 {
                elevations[z * side + x] = 200;
            }
        }
        assert!(ensure_seeded_island(&mut elevations, side, 7));
        let landmasses = derive_landmasses(&elevations, side);
        assert!(!landmasses.island_landmass_ids.is_empty());
    }

    #[test]
    fn downhill_chain_accumulates_flow_and_basin() {
        let side = 5;
        let mut elevations = vec![-10_i16; side * side];
        elevations[2 * side + 1] = 300;
        elevations[2 * side + 2] = 200;
        elevations[2 * side + 3] = 100;
        let hydrology = derive_hydrology(&elevations, side);
        assert_eq!(hydrology.flow_accumulation[2 * side + 3], 3);
        assert_eq!(hydrology.drainage_basin_ids[2 * side + 1], hydrology.drainage_basin_ids[2 * side + 3]);
    }
}
