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

    let land_effects = terrain
        .samples
        .iter()
        .zip(&effects.samples)
        .filter(|(sample, _)| sample.elevation_m >= terrain.sea_level_m)
        .map(|(_, effect)| effect)
        .collect::<Vec<_>>();

    let agriculture_min = land_effects
        .iter()
        .map(|effect| effect.agriculture_yield_permille)
        .min()
        .expect("land exists");
    let agriculture_max = land_effects
        .iter()
        .map(|effect| effect.agriculture_yield_permille)
        .max()
        .expect("land exists");
    let construction_min = land_effects
        .iter()
        .map(|effect| effect.construction_cost_permille)
        .min()
        .expect("land exists");
    let construction_max = land_effects
        .iter()
        .map(|effect| effect.construction_cost_permille)
        .max()
        .expect("land exists");
    let movement_min = land_effects
        .iter()
        .map(|effect| effect.movement_cost_permille)
        .min()
        .expect("land exists");
    let movement_max = land_effects
        .iter()
        .map(|effect| effect.movement_cost_permille)
        .max()
        .expect("land exists");
    let defense_min = land_effects
        .iter()
        .map(|effect| effect.defense_multiplier_permille)
        .min()
        .expect("land exists");
    let defense_max = land_effects
        .iter()
        .map(|effect| effect.defense_multiplier_permille)
        .max()
        .expect("land exists");
    let capacity_min = land_effects
        .iter()
        .map(|effect| effect.carrying_capacity_people_per_km2)
        .min()
        .expect("land exists");
    let capacity_max = land_effects
        .iter()
        .map(|effect| effect.carrying_capacity_people_per_km2)
        .max()
        .expect("land exists");
    let productivity_min = land_effects
        .iter()
        .map(|effect| effect.productivity_multiplier_permille)
        .min()
        .expect("land exists");
    let productivity_max = land_effects
        .iter()
        .map(|effect| effect.productivity_multiplier_permille)
        .max()
        .expect("land exists");

    assert!(agriculture_max > agriculture_min);
    assert!(construction_max > construction_min);
    assert!(movement_max > movement_min);
    assert!(defense_max > defense_min);
    assert!(capacity_max > capacity_min);
    assert!(productivity_max > productivity_min);
    assert!(land_effects
        .iter()
        .any(|effect| effect.port_feasibility_permille > 0));
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
        .filter_map(|(index, sample)| (sample.elevation_m >= 0).then_some(index))
        .take(25)
        .collect::<Vec<_>>();

    let aggregate = effects
        .aggregate_indices(&indices)
        .expect("non-empty valid indices aggregate");
    assert!(aggregate.construction_cost_permille >= 1_000);
    assert!(aggregate.movement_cost_permille >= 1_000);
    assert!(aggregate.carrying_capacity_people_per_km2 > 0);
}
