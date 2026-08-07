#![forbid(unsafe_code)]

use std::time::Duration;

use simulation_core::{SimulationClock, SimulationEngine, SimulationSpeed};
use simulation_model::{CommandPayload, CommandTiming, CountryId, SimulationDate};

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

    println!(
        "date={:04}-{:02}-{:02} elapsed_days={} ticks={} commands={} render_stride={} digest={:016x}",
        engine.state().date.year,
        engine.state().date.month,
        engine.state().date.day,
        engine.state().elapsed_days,
        report.ticks_executed,
        report.commands_executed,
        report.render_stride_days,
        engine.state_digest()
    );
}
