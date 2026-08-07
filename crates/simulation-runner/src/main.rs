#![forbid(unsafe_code)]

use std::time::Duration;

use simulation_core::{SimulationClock, SimulationEngine, SimulationSpeed};
use simulation_model::{CommandPayload, CommandTiming, CountryId, SimulationDate, TerrainState};
use simulation_save::{create_bundle, SaveKind};
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
    let restored_terrain = authoritative_terrain(&restored);
    let hydrology = restored_terrain.derive_hydrology();
    let landmasses = restored_terrain.derive_landmasses();
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

    println!(
        "date={:04}-{:02}-{:02} elapsed_days={} ticks={} commands={} events={} journal={} render_stride={} terrain_samples={} land_samples={} river_samples={} drainage_basins={} islands={} save_json_bytes={} save_state_bytes={} canonical_digest={:016x} random_probe={:016x}",
        restored.state().date.year,
        restored.state().date.month,
        restored.state().date.day,
        restored.state().elapsed_days,
        report.ticks_executed,
        report.commands_executed,
        report.events_emitted,
        journal.len(),
        report.render_stride_days,
        restored_terrain.samples.len(),
        restored_terrain.land_sample_count(),
        river_samples,
        drainage_basins,
        landmasses.island_landmass_ids.len(),
        bundle.metadata_json.len(),
        bundle.state_binary.len(),
        canonical_digest,
        random_probe
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
