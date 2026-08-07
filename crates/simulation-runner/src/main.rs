#![forbid(unsafe_code)]

use std::time::Duration;

use simulation_core::{SimulationClock, SimulationEngine, SimulationSpeed, TickReport};
use simulation_model::{
    derive_terrain_effects, CommandPayload, CommandTiming, CountryId, SimulationDate, TerrainState,
};
use simulation_save::{create_bundle, SaveBundle, SaveKind};
use simulation_worldgen::generate_trial_terrain;

fn main() {
    let seed = 1;
    let mut engine = initialized_engine(seed);
    engine
        .submit_user_command(CommandTiming::Immediate, CommandPayload::NoOp { token: 1 })
        .expect("trial user command must be valid");
    engine
        .submit_country_ai_command(
            CountryId(1),
            CommandTiming::Scheduled(SimulationDate::new(1, 1, 10)),
            CommandPayload::NoOp { token: 2 },
        )
        .expect("trial AI command must be valid");

    let mut clock = SimulationClock::new(SimulationSpeed::X365);
    let report = clock.advance_real_time(&mut engine, Duration::from_secs(1));
    let original_terrain = authoritative_terrain(&engine);

    let bundle = engine
        .save_bundle(SaveKind::Manual)
        .expect("headless state must be saveable");
    let restored =
        SimulationEngine::from_save_bundle(&bundle).expect("headless save must restore exactly");
    assert_eq!(restored.export_snapshot(), engine.export_snapshot());
    assert_same_derived_geography(original_terrain, authoritative_terrain(&restored));

    let journal = engine.accepted_command_journal();
    let replayed_without_worldgen = SimulationEngine::replay_from_journal(
        engine.state().seed,
        engine.state().elapsed_days,
        &journal,
    )
    .expect("headless command journal must replay");
    let mut replay_snapshot = replayed_without_worldgen.export_snapshot();
    replay_snapshot.world.terrain =
        Some(generate_trial_terrain(engine.state().seed).expect("replay terrain must regenerate"));
    let replay_bundle =
        create_bundle(&replay_snapshot, SaveKind::Manual).expect("replay snapshot must save");
    let replayed =
        SimulationEngine::from_save_bundle(&replay_bundle).expect("replay snapshot must restore");
    assert_eq!(replayed.export_snapshot(), engine.export_snapshot());
    assert_same_derived_geography(original_terrain, authoritative_terrain(&replayed));

    let canonical_digest = restored
        .authoritative_state_digest()
        .expect("authoritative digest must be valid");
    assert_eq!(
        canonical_digest,
        replayed
            .authoritative_state_digest()
            .expect("replayed digest must be valid")
    );
    let random_probe = restored.deterministic_random_u64(0x4741_4f4e, 0);

    print_headless_summary(
        &restored,
        &bundle,
        &report,
        journal.len(),
        canonical_digest,
        random_probe,
    );
}

fn initialized_engine(seed: u64) -> SimulationEngine {
    let mut initial = SimulationEngine::new(seed).export_snapshot();
    initial.world.terrain =
        Some(generate_trial_terrain(seed).expect("trial terrain must generate"));
    let bundle =
        create_bundle(&initial, SaveKind::Manual).expect("trial initial state must be saveable");
    SimulationEngine::from_save_bundle(&bundle)
        .expect("trial initial state must restore into the engine")
}

fn authoritative_terrain(engine: &SimulationEngine) -> &TerrainState {
    engine
        .state()
        .terrain
        .as_ref()
        .expect("terrain is authoritative")
}

fn assert_same_derived_geography(expected: &TerrainState, actual: &TerrainState) {
    assert_eq!(actual.derive_hydrology(), expected.derive_hydrology());
    assert_eq!(actual.derive_landmasses(), expected.derive_landmasses());
}

fn print_headless_summary(
    restored: &SimulationEngine,
    bundle: &SaveBundle,
    report: &TickReport,
    journal_len: usize,
    canonical_digest: u64,
    random_probe: u64,
) {
    let terrain = authoritative_terrain(restored);
    let hydrology = terrain.derive_hydrology();
    let landmasses = terrain.derive_landmasses();
    let river_samples = hydrology
        .river_orders
        .iter()
        .filter(|&&order| order > 0)
        .count();
    let drainage_basins = hydrology
        .drainage_basin_ids
        .iter()
        .copied()
        .max()
        .unwrap_or(0);
    let effects = summarize_terrain_effects(terrain);

    println!(
        "date={:04}-{:02}-{:02} elapsed_days={} ticks={} commands={} events={} journal={} render_stride={} terrain_samples={} land_samples={} river_samples={} drainage_basins={} islands={} agriculture={}..{} construction={}..{} movement={}..{} defense={}..{} port_candidates={} carrying={}..{} productivity={}..{} save_json_bytes={} save_state_bytes={} canonical_digest={:016x} random_probe={:016x}",
        restored.state().date.year,
        restored.state().date.month,
        restored.state().date.day,
        restored.state().elapsed_days,
        report.ticks_executed,
        report.commands_executed,
        report.events_emitted,
        journal_len,
        report.render_stride_days,
        terrain.samples.len(),
        terrain.land_sample_count(),
        river_samples,
        drainage_basins,
        landmasses.island_landmass_ids.len(),
        effects.agriculture_min,
        effects.agriculture_max,
        effects.construction_min,
        effects.construction_max,
        effects.movement_min,
        effects.movement_max,
        effects.defense_min,
        effects.defense_max,
        effects.port_candidates,
        effects.carrying_min,
        effects.carrying_max,
        effects.productivity_min,
        effects.productivity_max,
        bundle.metadata_json.len(),
        bundle.state_binary.len(),
        canonical_digest,
        random_probe
    );
}

#[derive(Debug, Clone, Copy)]
struct TerrainEffectSummary {
    agriculture_min: u16,
    agriculture_max: u16,
    construction_min: u16,
    construction_max: u16,
    movement_min: u16,
    movement_max: u16,
    defense_min: u16,
    defense_max: u16,
    port_candidates: usize,
    carrying_min: u16,
    carrying_max: u16,
    productivity_min: u16,
    productivity_max: u16,
}

fn summarize_terrain_effects(terrain: &TerrainState) -> TerrainEffectSummary {
    let field = derive_terrain_effects(terrain);
    let mut summary = TerrainEffectSummary {
        agriculture_min: u16::MAX,
        agriculture_max: 0,
        construction_min: u16::MAX,
        construction_max: 0,
        movement_min: u16::MAX,
        movement_max: 0,
        defense_min: u16::MAX,
        defense_max: 0,
        port_candidates: 0,
        carrying_min: u16::MAX,
        carrying_max: 0,
        productivity_min: u16::MAX,
        productivity_max: 0,
    };
    let mut land_count = 0_usize;

    for (sample, effect) in terrain.samples.iter().zip(&field.samples) {
        if sample.elevation_m < terrain.sea_level_m {
            continue;
        }
        land_count += 1;
        summary.agriculture_min = summary
            .agriculture_min
            .min(effect.agriculture_yield_permille);
        summary.agriculture_max = summary
            .agriculture_max
            .max(effect.agriculture_yield_permille);
        summary.construction_min = summary
            .construction_min
            .min(effect.construction_cost_permille);
        summary.construction_max = summary
            .construction_max
            .max(effect.construction_cost_permille);
        summary.movement_min = summary.movement_min.min(effect.movement_cost_permille);
        summary.movement_max = summary.movement_max.max(effect.movement_cost_permille);
        summary.defense_min = summary.defense_min.min(effect.defense_multiplier_permille);
        summary.defense_max = summary.defense_max.max(effect.defense_multiplier_permille);
        if effect.port_feasibility_permille > 0 {
            summary.port_candidates += 1;
        }
        summary.carrying_min = summary
            .carrying_min
            .min(effect.carrying_capacity_people_per_km2);
        summary.carrying_max = summary
            .carrying_max
            .max(effect.carrying_capacity_people_per_km2);
        summary.productivity_min = summary
            .productivity_min
            .min(effect.productivity_multiplier_permille);
        summary.productivity_max = summary
            .productivity_max
            .max(effect.productivity_multiplier_permille);
    }

    if land_count == 0 {
        summary.agriculture_min = 0;
        summary.construction_min = 0;
        summary.movement_min = 0;
        summary.defense_min = 0;
        summary.carrying_min = 0;
        summary.productivity_min = 0;
    }
    summary
}
