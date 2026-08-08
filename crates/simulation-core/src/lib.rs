#![forbid(unsafe_code)]

mod entity_lifecycle;
mod event_ledger;
mod population_survival;

use std::time::Duration;

use event_ledger::{EventDraft, EventLedger};
use simulation_model::{
    CommandError, CommandExecutionRecord, CommandId, CommandPayload, CommandPriority,
    CommandRequest, CommandSource, CommandTiming, CountryId, DateBoundary, EventCategory,
    EventFilter, EventPayload, EventRecord, EventSource, QueuedCommand, SimulationDate, WorldState,
};

const NANOS_PER_SECOND: u128 = 1_000_000_000;
const TARGET_RENDER_SAMPLES_PER_SECOND: u32 = 30;

/// User-selectable simulation speed. Values are simulated days per real second.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SimulationSpeed {
    #[default]
    Paused,
    X1,
    X7,
    X30,
    X90,
    X365,
}

impl SimulationSpeed {
    #[must_use]
    pub const fn days_per_second(self) -> u32 {
        match self {
            Self::Paused => 0,
            Self::X1 => 1,
            Self::X7 => 7,
            Self::X30 => 30,
            Self::X90 => 90,
            Self::X365 => 365,
        }
    }

    /// Suggested number of simulated days between visible render samples.
    ///
    /// Logic always executes every day. Only presentation samples are reduced
    /// at high speed, keeping the simulation deterministic while preventing the
    /// renderer from trying to display hundreds of daily frames per second.
    #[must_use]
    pub const fn render_stride_days(self) -> u32 {
        let days = self.days_per_second();
        if days <= TARGET_RENDER_SAMPLES_PER_SECOND {
            return 1;
        }
        days.div_ceil(TARGET_RENDER_SAMPLES_PER_SECOND)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TickReport {
    pub boundary: DateBoundary,
    pub commands_executed: u64,
    pub events_emitted: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AdvanceReport {
    pub ticks_executed: u64,
    pub commands_executed: u64,
    pub events_emitted: u64,
    pub month_boundaries: u64,
    pub quarter_boundaries: u64,
    pub year_boundaries: u64,
    pub render_stride_days: u32,
}

impl AdvanceReport {
    fn observe_tick(&mut self, tick: TickReport) {
        self.commands_executed += tick.commands_executed;
        self.events_emitted += tick.events_emitted;
        self.month_boundaries += u64::from(tick.boundary.month_changed);
        self.quarter_boundaries += u64::from(tick.boundary.quarter_changed);
        self.year_boundaries += u64::from(tick.boundary.year_changed);
    }
}

/// Deterministic fixed-tick simulation kernel.
///
/// The engine is the sole owner of authoritative simulation state, the command
/// queue, command execution history and Event Ledger. UI and future country AI
/// layers submit requests; only the engine resolves and applies them.
#[derive(Debug, Clone)]
pub struct SimulationEngine {
    state: WorldState,
    pending_commands: Vec<QueuedCommand>,
    executed_commands: Vec<CommandExecutionRecord>,
    event_ledger: EventLedger,
    next_command_id: u64,
}

impl SimulationEngine {
    #[must_use]
    pub const fn new(seed: u64) -> Self {
        Self {
            state: WorldState::new(seed),
            pending_commands: Vec::new(),
            executed_commands: Vec::new(),
            event_ledger: EventLedger::new(),
            next_command_id: 1,
        }
    }

    #[must_use]
    pub const fn state(&self) -> &WorldState {
        &self.state
    }

    #[must_use]
    pub fn pending_commands(&self) -> &[QueuedCommand] {
        &self.pending_commands
    }

    #[must_use]
    pub fn command_log(&self) -> &[CommandExecutionRecord] {
        &self.executed_commands
    }

    /// Returns the complete immutable Event Ledger in canonical append order.
    #[must_use]
    pub fn event_log(&self) -> &[EventRecord] {
        self.event_ledger.records()
    }

    /// Filters immutable Event Ledger records without mutating simulation state.
    pub fn query_events(&self, filter: EventFilter) -> impl Iterator<Item = &EventRecord> {
        self.event_ledger
            .records()
            .iter()
            .filter(move |event| filter.matches(event))
    }

    /// Accepts a command request after deterministic validation and assigns its
    /// canonical identity, execution date and priority lane.
    ///
    /// # Errors
    ///
    /// Returns [`CommandError`] when a scheduled date is invalid, is already in
    /// the past, or no further deterministic command identifiers can be issued.
    pub fn submit_command(&mut self, request: CommandRequest) -> Result<CommandId, CommandError> {
        if request.payload.is_entity_lifecycle() && request.source != CommandSource::System {
            return Err(CommandError::InvalidSourceForPayload);
        }
        let execute_on = self.resolve_execution_date(request.timing)?;
        let id = CommandId(self.next_command_id);
        self.next_command_id = self
            .next_command_id
            .checked_add(1)
            .ok_or(CommandError::CommandIdExhausted)?;

        let command = QueuedCommand {
            id,
            source: request.source,
            submitted_on: self.state.date,
            execute_on,
            priority: CommandPriority::for_source(request.source),
            payload: request.payload,
        };

        self.pending_commands.push(command);
        self.pending_commands
            .sort_by_key(|queued| (queued.execute_on, queued.priority, queued.id));

        Ok(id)
    }

    /// Submits a system-originated deterministic lifecycle command.
    ///
    /// # Errors
    /// Returns [`CommandError`] under the same scheduling rules as other commands.
    pub fn submit_system_command(
        &mut self,
        timing: CommandTiming,
        payload: CommandPayload,
    ) -> Result<CommandId, CommandError> {
        self.submit_command(CommandRequest::system(timing, payload))
    }

    /// Submits a user-originated command using the user intervention priority.
    ///
    /// # Errors
    ///
    /// Returns [`CommandError`] under the same validation rules as
    /// [`Self::submit_command`].
    pub fn submit_user_command(
        &mut self,
        timing: CommandTiming,
        payload: CommandPayload,
    ) -> Result<CommandId, CommandError> {
        self.submit_command(CommandRequest::user(timing, payload))
    }

    /// Submits an autonomous country-AI command using the AI priority lane.
    ///
    /// # Errors
    ///
    /// Returns [`CommandError`] under the same validation rules as
    /// [`Self::submit_command`].
    pub fn submit_country_ai_command(
        &mut self,
        country_id: CountryId,
        timing: CommandTiming,
        payload: CommandPayload,
    ) -> Result<CommandId, CommandError> {
        self.submit_command(CommandRequest::country_ai(country_id, timing, payload))
    }

    fn resolve_execution_date(
        &self,
        timing: CommandTiming,
    ) -> Result<SimulationDate, CommandError> {
        match timing {
            CommandTiming::Immediate => Ok(self.state.date),
            CommandTiming::Scheduled(requested) => {
                if !requested.is_valid() {
                    return Err(CommandError::InvalidScheduledDate { requested });
                }
                if requested < self.state.date {
                    return Err(CommandError::ScheduledInPast {
                        requested,
                        current: self.state.date,
                    });
                }
                Ok(requested)
            }
        }
    }

    fn execute_due_commands(&mut self) -> u64 {
        let current_date = self.state.date;
        let due_count = self
            .pending_commands
            .partition_point(|command| command.execute_on <= current_date);
        if due_count == 0 {
            return 0;
        }

        let due_commands: Vec<_> = self.pending_commands.drain(..due_count).collect();
        let executed = u64::try_from(due_commands.len()).unwrap_or(u64::MAX);

        for command in due_commands {
            self.execute_command(command, current_date);
        }

        executed
    }

    fn execute_command(&mut self, command: QueuedCommand, executed_on: SimulationDate) {
        let command_id = command.id;
        let command_source = command.source;

        let lifecycle_result = self.apply_entity_payload(&command.payload);

        self.executed_commands.push(CommandExecutionRecord {
            command,
            executed_on,
        });

        match lifecycle_result {
            Ok(Some(payload)) => {
                self.event_ledger.append(
                    executed_on,
                    self.state.elapsed_days,
                    EventDraft {
                        category: EventCategory::EntityLifecycle,
                        source: EventSource::System,
                        payload,
                    },
                );
            }
            Ok(None) => {}
            Err(error) => {
                self.event_ledger.append(
                    executed_on,
                    self.state.elapsed_days,
                    EventDraft {
                        category: EventCategory::EntityLifecycle,
                        source: EventSource::System,
                        payload: EventPayload::EntityMutationRejected { error },
                    },
                );
            }
        }

        let (category, source) = match command_source {
            CommandSource::System => (EventCategory::System, EventSource::System),
            CommandSource::User => (EventCategory::UserIntervention, EventSource::User),
            CommandSource::CountryAi(country_id) => {
                (EventCategory::System, EventSource::Country(country_id))
            }
        };

        self.event_ledger.append(
            executed_on,
            self.state.elapsed_days,
            EventDraft {
                category,
                source,
                payload: EventPayload::CommandExecuted { command_id },
            },
        );
    }

    /// Advances exactly one simulated day.
    ///
    /// Commands effective on the current date are executed first, in the fixed
    /// order `(date, priority, command id)`. HumanGroup population/survival is
    /// then advanced once from authoritative runtime state before the calendar
    /// moves to the next day. This keeps demographic outcomes independent of
    /// renderer cadence and real-time playback speed.
    pub fn tick(&mut self) -> TickReport {
        let events_before = self.event_ledger.records().len();
        let commands_executed = self.execute_due_commands();
        let _population_report = population_survival::advance_population_survival(&mut self.state);
        let events_emitted = self
            .event_ledger
            .records()
            .len()
            .saturating_sub(events_before);
        let events_emitted = u64::try_from(events_emitted).unwrap_or(u64::MAX);

        let boundary = self.state.date.advance_one_day();
        self.state.elapsed_days += 1;
        TickReport {
            boundary,
            commands_executed,
            events_emitted,
        }
    }

    /// Advances an explicit deterministic number of days.
    pub fn advance_days(&mut self, days: u64) -> AdvanceReport {
        let mut report = AdvanceReport {
            render_stride_days: 1,
            ..AdvanceReport::default()
        };

        for _ in 0..days {
            report.observe_tick(self.tick());
        }
        report.ticks_executed = days;
        report
    }

    /// Manual one-day step used by the UI while paused.
    pub fn step_one_day(&mut self) -> AdvanceReport {
        self.advance_days(1)
    }

    /// Manual one-month control defined by the trial plan as exactly 30 days.
    pub fn step_one_month(&mut self) -> AdvanceReport {
        self.advance_days(30)
    }

    /// Returns a deterministic FNV-1a digest of the current minimal world
    /// state. Stage 1.6 will extend canonical hashing to all authoritative
    /// dynamic state and command/event/replay positions.
    #[must_use]
    pub fn state_digest(&self) -> u64 {
        let mut hash = 0xcbf2_9ce4_8422_2325_u64;
        for byte in self
            .state
            .seed
            .to_le_bytes()
            .into_iter()
            .chain(self.state.elapsed_days.to_le_bytes())
            .chain(self.state.date.year.to_le_bytes())
            .chain([self.state.date.month, self.state.date.day])
        {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        hash
    }
}

/// Converts real elapsed time into an integer number of simulation ticks.
///
/// The fractional accumulator uses integer nanoseconds; wall-clock timing does
/// not enter `WorldState`, so playback speed changes cannot alter the result of
/// a given tick sequence.
#[derive(Debug, Clone, Default)]
pub struct SimulationClock {
    speed: SimulationSpeed,
    scaled_nanosecond_remainder: u128,
}

impl SimulationClock {
    #[must_use]
    pub const fn new(speed: SimulationSpeed) -> Self {
        Self {
            speed,
            scaled_nanosecond_remainder: 0,
        }
    }

    #[must_use]
    pub const fn speed(&self) -> SimulationSpeed {
        self.speed
    }

    pub fn set_speed(&mut self, speed: SimulationSpeed) {
        self.speed = speed;
        if speed == SimulationSpeed::Paused {
            self.scaled_nanosecond_remainder = 0;
        }
    }

    pub fn advance_real_time(
        &mut self,
        engine: &mut SimulationEngine,
        elapsed: Duration,
    ) -> AdvanceReport {
        let days_per_second = self.speed.days_per_second();
        if days_per_second == 0 {
            return AdvanceReport {
                render_stride_days: self.speed.render_stride_days(),
                ..AdvanceReport::default()
            };
        }

        let scaled = elapsed
            .as_nanos()
            .saturating_mul(u128::from(days_per_second))
            .saturating_add(self.scaled_nanosecond_remainder);
        let due_ticks = scaled / NANOS_PER_SECOND;
        self.scaled_nanosecond_remainder = scaled % NANOS_PER_SECOND;

        let due_ticks = u64::try_from(due_ticks).unwrap_or(u64::MAX);
        let mut report = engine.advance_days(due_ticks);
        report.render_stride_days = self.speed.render_stride_days();
        report
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use simulation_model::{
        CommandError, CommandPayload, CommandSource, CommandTiming, CountryId, EventCategory,
        EventFilter, EventPayload, EventSource, SimulationDate,
    };

    use super::{SimulationClock, SimulationEngine, SimulationSpeed};

    #[test]
    fn one_tick_advances_one_day() {
        let mut engine = SimulationEngine::new(7);
        engine.tick();
        assert_eq!(engine.state().elapsed_days, 1);
        assert_eq!(engine.state().date.day, 2);
    }

    #[test]
    fn manual_month_is_exactly_thirty_days() {
        let mut engine = SimulationEngine::new(7);
        let report = engine.step_one_month();
        assert_eq!(report.ticks_executed, 30);
        assert_eq!(engine.state().elapsed_days, 30);
        assert_eq!(engine.state().date.month, 1);
        assert_eq!(engine.state().date.day, 31);
    }

    #[test]
    fn pause_executes_no_ticks() {
        let mut engine = SimulationEngine::new(1);
        let mut clock = SimulationClock::new(SimulationSpeed::Paused);
        let report = clock.advance_real_time(&mut engine, Duration::from_secs(10));
        assert_eq!(report.ticks_executed, 0);
        assert_eq!(engine.state().elapsed_days, 0);
    }

    #[test]
    fn speed_modes_map_to_expected_daily_ticks() {
        for (speed, expected) in [
            (SimulationSpeed::X1, 1),
            (SimulationSpeed::X7, 7),
            (SimulationSpeed::X30, 30),
            (SimulationSpeed::X90, 90),
            (SimulationSpeed::X365, 365),
        ] {
            let mut engine = SimulationEngine::new(1);
            let mut clock = SimulationClock::new(speed);
            let report = clock.advance_real_time(&mut engine, Duration::from_secs(1));
            assert_eq!(report.ticks_executed, expected);
        }
    }

    #[test]
    fn fractional_real_time_accumulates_without_float_drift() {
        let mut engine = SimulationEngine::new(1);
        let mut clock = SimulationClock::new(SimulationSpeed::X7);
        for _ in 0..10 {
            clock.advance_real_time(&mut engine, Duration::from_millis(100));
        }
        assert_eq!(engine.state().elapsed_days, 7);
    }

    #[test]
    fn rendering_is_sampled_but_logic_is_not_skipped() {
        assert_eq!(SimulationSpeed::X30.render_stride_days(), 1);
        assert_eq!(SimulationSpeed::X90.render_stride_days(), 3);
        assert_eq!(SimulationSpeed::X365.render_stride_days(), 13);

        let mut engine = SimulationEngine::new(1);
        let mut clock = SimulationClock::new(SimulationSpeed::X365);
        let report = clock.advance_real_time(&mut engine, Duration::from_secs(1));
        assert_eq!(report.ticks_executed, 365);
        assert_eq!(report.render_stride_days, 13);
    }

    #[test]
    fn same_seed_and_ticks_produce_same_digest() {
        let mut left = SimulationEngine::new(99);
        let mut right = SimulationEngine::new(99);
        left.advance_days(10_000);
        right.advance_days(10_000);
        assert_eq!(left.state_digest(), right.state_digest());
    }

    #[test]
    fn playback_speed_does_not_change_tick_result() {
        let mut manual = SimulationEngine::new(2026);
        manual.advance_days(365);

        let mut realtime = SimulationEngine::new(2026);
        let mut clock = SimulationClock::new(SimulationSpeed::X365);
        clock.advance_real_time(&mut realtime, Duration::from_secs(1));

        assert_eq!(manual.state_digest(), realtime.state_digest());
    }

    #[test]
    fn immediate_user_command_executes_on_current_simulation_date() {
        let mut engine = SimulationEngine::new(1);
        let command_id = engine
            .submit_user_command(CommandTiming::Immediate, CommandPayload::NoOp { token: 11 })
            .expect("valid immediate command");

        assert_eq!(engine.pending_commands().len(), 1);
        let report = engine.step_one_day();
        assert_eq!(report.commands_executed, 1);
        assert!(engine.pending_commands().is_empty());
        assert_eq!(engine.command_log().len(), 1);
        assert_eq!(engine.command_log()[0].command.id, command_id);
        assert_eq!(engine.command_log()[0].executed_on, SimulationDate::START);
    }

    #[test]
    fn scheduled_command_waits_until_its_effective_date() {
        let mut engine = SimulationEngine::new(1);
        engine
            .submit_user_command(
                CommandTiming::Scheduled(SimulationDate::new(1, 1, 3)),
                CommandPayload::NoOp { token: 30 },
            )
            .expect("future command should be accepted");

        let first = engine.step_one_day();
        let second = engine.step_one_day();
        let third = engine.step_one_day();

        assert_eq!(first.commands_executed, 0);
        assert_eq!(second.commands_executed, 0);
        assert_eq!(third.commands_executed, 1);
        assert_eq!(
            engine.command_log()[0].executed_on,
            SimulationDate::new(1, 1, 3)
        );
    }

    #[test]
    fn user_commands_precede_country_ai_commands_on_the_same_date() {
        let mut engine = SimulationEngine::new(1);
        let ai = engine
            .submit_country_ai_command(
                CountryId(4),
                CommandTiming::Immediate,
                CommandPayload::NoOp { token: 1 },
            )
            .expect("valid AI command");
        let user = engine
            .submit_user_command(CommandTiming::Immediate, CommandPayload::NoOp { token: 2 })
            .expect("valid user command");

        engine.step_one_day();

        assert_eq!(engine.command_log().len(), 2);
        assert_eq!(engine.command_log()[0].command.id, user);
        assert_eq!(engine.command_log()[0].command.source, CommandSource::User);
        assert_eq!(engine.command_log()[1].command.id, ai);
        assert_eq!(
            engine.command_log()[1].command.source,
            CommandSource::CountryAi(CountryId(4))
        );
    }

    #[test]
    fn equal_priority_commands_preserve_submission_order() {
        let mut engine = SimulationEngine::new(1);
        let first = engine
            .submit_user_command(CommandTiming::Immediate, CommandPayload::NoOp { token: 1 })
            .expect("valid user command");
        let second = engine
            .submit_user_command(CommandTiming::Immediate, CommandPayload::NoOp { token: 2 })
            .expect("valid user command");

        engine.step_one_day();

        assert_eq!(engine.command_log()[0].command.id, first);
        assert_eq!(engine.command_log()[1].command.id, second);
    }

    #[test]
    fn invalid_and_past_scheduled_dates_are_rejected_without_queue_mutation() {
        let mut engine = SimulationEngine::new(1);
        let invalid = SimulationDate::new(1, 2, 29);
        assert_eq!(
            engine.submit_user_command(
                CommandTiming::Scheduled(invalid),
                CommandPayload::NoOp { token: 1 },
            ),
            Err(CommandError::InvalidScheduledDate { requested: invalid })
        );

        engine.advance_days(5);
        let past = SimulationDate::new(1, 1, 2);
        assert_eq!(
            engine.submit_user_command(
                CommandTiming::Scheduled(past),
                CommandPayload::NoOp { token: 2 },
            ),
            Err(CommandError::ScheduledInPast {
                requested: past,
                current: SimulationDate::new(1, 1, 6),
            })
        );
        assert!(engine.pending_commands().is_empty());
    }

    #[test]
    fn command_execution_creates_immutable_user_intervention_event() {
        let mut engine = SimulationEngine::new(1);
        let command_id = engine
            .submit_user_command(CommandTiming::Immediate, CommandPayload::NoOp { token: 50 })
            .expect("valid user command");

        let report = engine.step_one_day();
        assert_eq!(report.events_emitted, 1);
        assert_eq!(engine.event_log().len(), 1);

        let event = engine.event_log()[0];
        assert_eq!(event.occurred_on, SimulationDate::START);
        assert_eq!(event.tick_index, 0);
        assert_eq!(event.category, EventCategory::UserIntervention);
        assert_eq!(event.source, EventSource::User);
        assert_eq!(event.payload, EventPayload::CommandExecuted { command_id });
    }

    #[test]
    fn country_ai_command_records_country_source_without_inventing_domain_event() {
        let mut engine = SimulationEngine::new(1);
        engine
            .submit_country_ai_command(
                CountryId(6),
                CommandTiming::Immediate,
                CommandPayload::NoOp { token: 60 },
            )
            .expect("valid AI command");

        engine.step_one_day();
        let event = engine.event_log()[0];
        assert_eq!(event.category, EventCategory::System);
        assert_eq!(event.source, EventSource::Country(CountryId(6)));
    }

    #[test]
    fn event_query_filters_by_category_source_and_date() {
        let mut engine = SimulationEngine::new(1);
        engine
            .submit_user_command(CommandTiming::Immediate, CommandPayload::NoOp { token: 1 })
            .expect("valid user command");
        engine
            .submit_country_ai_command(
                CountryId(3),
                CommandTiming::Scheduled(SimulationDate::new(1, 1, 2)),
                CommandPayload::NoOp { token: 2 },
            )
            .expect("valid AI command");
        engine.advance_days(2);

        let user_events = engine
            .query_events(EventFilter {
                category: Some(EventCategory::UserIntervention),
                ..EventFilter::default()
            })
            .count();
        assert_eq!(user_events, 1);

        let country_events = engine
            .query_events(EventFilter {
                source: Some(EventSource::Country(CountryId(3))),
                from_date: Some(SimulationDate::new(1, 1, 2)),
                through_date: Some(SimulationDate::new(1, 1, 2)),
                ..EventFilter::default()
            })
            .count();
        assert_eq!(country_events, 1);
    }

    #[test]
    fn command_and_event_order_are_identical_at_manual_and_high_speed_playback() {
        fn seed_commands(engine: &mut SimulationEngine) {
            engine
                .submit_country_ai_command(
                    CountryId(2),
                    CommandTiming::Scheduled(SimulationDate::new(1, 1, 3)),
                    CommandPayload::NoOp { token: 10 },
                )
                .expect("valid AI command");
            engine
                .submit_user_command(
                    CommandTiming::Scheduled(SimulationDate::new(1, 1, 3)),
                    CommandPayload::NoOp { token: 20 },
                )
                .expect("valid user command");
            engine
                .submit_user_command(CommandTiming::Immediate, CommandPayload::NoOp { token: 30 })
                .expect("valid immediate command");
        }

        let mut manual = SimulationEngine::new(77);
        seed_commands(&mut manual);
        manual.advance_days(7);

        let mut realtime = SimulationEngine::new(77);
        seed_commands(&mut realtime);
        let mut clock = SimulationClock::new(SimulationSpeed::X7);
        clock.advance_real_time(&mut realtime, Duration::from_secs(1));

        assert_eq!(manual.command_log(), realtime.command_log());
        assert_eq!(manual.event_log(), realtime.event_log());
        assert_eq!(manual.state_digest(), realtime.state_digest());
    }
}
