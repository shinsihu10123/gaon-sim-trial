use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::{TerrainHydrology, TerrainLandmasses, TerrainState, NO_DOWNSTREAM_INDEX};

const RIVER_HEADWATER_THRESHOLD: u32 = 8;
const RIVER_MEDIUM_THRESHOLD: u32 = 32;
const RIVER_MAJOR_THRESHOLD: u32 = 128;

/// Derives deterministic downstream routing, drainage basins, flow accumulation
/// and river order from a canonical terrain heightfield.
#[must_use]
pub fn derive_terrain_hydrology(terrain: &TerrainState) -> TerrainHydrology {
    let elevations = terrain
        .samples
        .iter()
        .map(|sample| sample.elevation_m)
        .collect::<Vec<_>>();
    derive_hydrology_from_elevations(&elevations, usize::from(terrain.width))
}

/// Derives deterministic connected landmasses and island identifiers from a
/// canonical terrain heightfield. The largest connected landmass is treated as
/// the mainland; every other disconnected landmass is classified as an island.
#[must_use]
pub fn derive_terrain_landmasses(terrain: &TerrainState) -> TerrainLandmasses {
    let elevations = terrain
        .samples
        .iter()
        .map(|sample| sample.elevation_m)
        .collect::<Vec<_>>();
    derive_landmasses_from_elevations(&elevations, usize::from(terrain.width))
}

fn derive_landmasses_from_elevations(elevations: &[i16], side: usize) -> TerrainLandmasses {
    let mut landmass_ids = vec![0_u16; elevations.len()];
    let mut component_sizes = Vec::new();
    let mut next_id = 1_u16;

    for start in 0..elevations.len() {
        if elevations[start] < 0 || landmass_ids[start] != 0 {
            continue;
        }

        let id = next_id;
        next_id = next_id.saturating_add(1);
        let mut component_size = 0_usize;
        let mut queue = VecDeque::from([start]);
        landmass_ids[start] = id;

        while let Some(index) = queue.pop_front() {
            component_size += 1;
            let x = index % side;
            let z = index / side;
            for neighbor in neighbor_indices(x, z, side) {
                if elevations[neighbor] >= 0 && landmass_ids[neighbor] == 0 {
                    landmass_ids[neighbor] = id;
                    queue.push_back(neighbor);
                }
            }
        }
        component_sizes.push((id, component_size));
    }

    let mainland_id = component_sizes
        .iter()
        .max_by(|(left_id, left_size), (right_id, right_size)| {
            left_size
                .cmp(right_size)
                .then_with(|| right_id.cmp(left_id))
        })
        .map_or(0, |(id, _)| *id);
    let island_landmass_ids = component_sizes
        .iter()
        .filter_map(|(id, _)| (*id != mainland_id).then_some(*id))
        .collect();

    TerrainLandmasses {
        landmass_ids,
        island_landmass_ids,
    }
}

fn derive_hydrology_from_elevations(elevations: &[i16], side: usize) -> TerrainHydrology {
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
            let replaces_best = match best {
                None => true,
                Some((best_elevation, best_index)) => {
                    candidate < best_elevation
                        || (candidate == best_elevation && neighbor < best_index)
                }
            };
            if replaces_best {
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
        if elevations
            .get(downstream_index)
            .is_some_and(|&elevation| elevation >= 0)
        {
            flow_accumulation[downstream_index] =
                flow_accumulation[downstream_index].saturating_add(flow_accumulation[index]);
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
        let reaches_ocean_or_invalid = match elevations.get(next) {
            None => true,
            Some(&elevation) => elevation < 0,
        };
        if reaches_ocean_or_invalid {
            return next;
        }
        current = next;
    }
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

#[cfg(test)]
mod tests {
    use crate::{BiomeClass, ReliefClass, TerrainSample, TerrainState, TERRAIN_SAMPLE_COUNT};

    use super::{derive_terrain_hydrology, derive_terrain_landmasses};

    fn ocean_terrain() -> TerrainState {
        TerrainState::new_trial(vec![
            TerrainSample {
                elevation_m: -500,
                moisture_permille: 500,
                relief: ReliefClass::ShallowOcean,
                biome: BiomeClass::Ocean,
            };
            TERRAIN_SAMPLE_COUNT
        ])
        .expect("terrain valid")
    }

    #[test]
    fn ocean_has_no_land_hydrology_or_landmasses() {
        let terrain = ocean_terrain();
        let hydrology = derive_terrain_hydrology(&terrain);
        let landmasses = derive_terrain_landmasses(&terrain);
        assert!(hydrology.river_orders.iter().all(|&value| value == 0));
        assert!(landmasses.landmass_ids.iter().all(|&value| value == 0));
        assert!(landmasses.island_landmass_ids.is_empty());
    }
}
