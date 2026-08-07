#![forbid(unsafe_code)]

use std::time::Duration;

use simulation_core::{SimulationClock, SimulationEngine, SimulationSpeed};
use simulation_model::{CommandPayload, CommandTiming, CountryId, SimulationDate};
use simulation_save::SaveKind;

fn main() {
    let mut engine = SimulationEngine::new(1);
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

    let bundle = engine
        .save_bundle(SaveKind::Manual)
        .expect("headless state must be saveable");
    let restored =
        SimulationEngine::from_save_bundle(&bundle).expect("headless save must restore exactly");
    assert_eq!(restored.export_snapshot(), engine.export_snapshot());

    println!(
        "date={:04}-{:02}-{:02} elapsed_days={} ticks={} commands={} events={} render_stride={} save_json_bytes={} save_state_bytes={} digest={:016x}",
        restored.state().date.year,
        restored.state().date.month,
        restored.state().date.day,
        restored.state().elapsed_days,
        report.ticks_executed,
        report.commands_executed,
        report.events_emitted,
        report.render_stride_days,
        bundle.metadata_json.len(),
        bundle.state_binary.len(),
        restored.state_digest()
    );
}
