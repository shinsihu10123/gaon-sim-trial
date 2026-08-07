use simulation_model::{derive_terrain_effects, ReliefClass, TerrainEffects};
use simulation_worldgen::generate_trial_terrain;

#[test]
fn effect_field_is_deterministic_and_aligned_to_terrain() {
    for seed in [1, 2026, 2027, u64::MAX] {
        let first = generate_trial_terrain(seed).expect("terrain should generate");
        let second = generate_trial_terrain(seed).expect("terrain should generate");
        let first_effects = derive_terrain_effects(&first);
        let second_effects = derive_terrain_effects(&second);
        assert_eq!(first_effects, second_effects);
        assert_eq!(first_effects.samples.len(), first.samples.len());
    }
}

#[test]
fn ocean_has_no_land_economic_or_military_effects() {
    let terrain = generate_trial_terrain(2026).expect("terrain should generate");
    let effects = derive_terrain_effects(&terrain);
    for (sample, effect) in terrain.samples.iter().zip(&effects.samples) {
        if sample.elevation_m < terrain.sea_level_m {
            assert_eq!(*effect, TerrainEffects::OCEAN);
        }
    }
}

#[test]
fn generated_world_exposes_nontrivial_effect_ranges() {
    let terrain = generate_trial_terrain(2026).expect("terrain should generate");
    let effects = derive_terrain_effects(&terrain);

    let mut agriculture_min = u16::MAX;
    let mut agriculture_max = 0_u16;
    let mut construction_min = u16::MAX;
    let mut construction_max = 0_u16;
    let mut movement_min = u16::MAX;
    let mut movement_max = 0_u16;
    let mut defense_min = u16::MAX;
    let mut defense_max = 0_u16;
    let mut capacity_min = u16::MAX;
    let mut capacity_max = 0_u16;
    let mut productivity_min = u16::MAX;
    let mut productivity_max = 0_u16;
    let mut port_exists = false;
    let mut land_count = 0_usize;

    for (sample, effect) in terrain.samples.iter().zip(&effects.samples) {
        if sample.elevation_m < terrain.sea_level_m {
            continue;
        }
        land_count += 1;
        agriculture_min = agriculture_min.min(effect.agriculture_yield_permille);
        agriculture_max = agriculture_max.max(effect.agriculture_yield_permille);
        construction_min = construction_min.min(effect.construction_cost_permille);
        construction_max = construction_max.max(effect.construction_cost_permille);
        movement_min = movement_min.min(effect.movement_cost_permille);
        movement_max = movement_max.max(effect.movement_cost_permille);
        defense_min = defense_min.min(effect.defense_multiplier_permille);
        defense_max = defense_max.max(effect.defense_multiplier_permille);
        capacity_min = capacity_min.min(effect.carrying_capacity_people_per_km2);
        capacity_max = capacity_max.max(effect.carrying_capacity_people_per_km2);
        productivity_min = productivity_min.min(effect.productivity_multiplier_permille);
        productivity_max = productivity_max.max(effect.productivity_multiplier_permille);
        port_exists |= effect.port_feasibility_permille > 0;
    }

    assert!(land_count > 0);
    assert!(agriculture_max > agriculture_min);
    assert!(construction_max > construction_min);
    assert!(movement_max > movement_min);
    assert!(defense_max > defense_min);
    assert!(capacity_max > capacity_min);
    assert!(productivity_max > productivity_min);
    assert!(port_exists);
}

#[test]
fn mountains_trade_accessibility_for_defense() {
    let terrain = generate_trial_terrain(2026).expect("terrain should generate");
    let effects = derive_terrain_effects(&terrain);

    let plains = terrain
        .samples
        .iter()
        .zip(&effects.samples)
        .find(|(sample, _)| sample.relief == ReliefClass::Plains)
        .expect("plains exist")
        .1;
    let mountains = terrain
        .samples
        .iter()
        .zip(&effects.samples)
        .find(|(sample, _)| sample.relief == ReliefClass::Mountains)
        .expect("mountains exist")
        .1;

    assert!(mountains.construction_cost_permille > plains.construction_cost_permille);
    assert!(mountains.movement_cost_permille > plains.movement_cost_permille);
    assert!(mountains.defense_multiplier_permille > plains.defense_multiplier_permille);
}

#[test]
fn aggregation_produces_region_ready_average() {
    let terrain = generate_trial_terrain(2026).expect("terrain should generate");
    let effects = derive_terrain_effects(&terrain);
    let indices = terrain
        .samples
        .iter()
        .enumerate()
        .filter(|(_, sample)| sample.elevation_m >= 0)
        .map(|(index, _)| index)
        .take(25)
        .collect::<Vec<_>>();

    let aggregate = effects
        .aggregate_indices(&indices)
        .expect("non-empty valid indices aggregate");
    assert!(aggregate.construction_cost_permille >= 1_000);
    assert!(aggregate.movement_cost_permille >= 1_000);
    assert!(aggregate.carrying_capacity_people_per_km2 > 0);
}
