use simulation_model::{
    CommandRequest, CommandTiming, EventCategory, EventId, EventPayload, EventRecord, EventSource,
    QueuedCommand, SimulationDate,
};
use simulation_save::{
    checksum_fnv1a64, create_bundle, decode_bundle, EngineSnapshot, SaveBundle, SaveError, SaveKind,
};

use crate::SimulationEngine;

#[derive(Debug, Clone, Copy)]
pub(crate) struct EventDraft {
    pub category: EventCategory,
    pub source: EventSource,
    pub payload: EventPayload,
}

/// Append-only owner of immutable simulation facts.
#[derive(Debug, Clone)]
pub(crate) struct EventLedger {
    records: Vec<EventRecord>,
    next_event_id: u64,
}

impl EventLedger {
    pub(crate) const fn new() -> Self {
        Self {
            records: Vec::new(),
            next_event_id: 1,
        }
    }

    fn from_snapshot(records: Vec<EventRecord>, next_event_id: u64) -> Self {
        Self {
            records,
            next_event_id,
        }
    }

    pub(crate) fn records(&self) -> &[EventRecord] {
        &self.records
    }

    const fn next_event_id(&self) -> u64 {
        self.next_event_id
    }

    pub(crate) fn append(
        &mut self,
        occurred_on: SimulationDate,
        tick_index: u64,
        draft: EventDraft,
    ) -> EventId {
        let id = EventId(self.next_event_id);
        self.next_event_id = self
            .next_event_id
            .checked_add(1)
            .expect("event identifier space exhausted");

        self.records.push(EventRecord {
            id,
            occurred_on,
            tick_index,
            category: draft.category,
            source: draft.source,
            payload: draft.payload,
        });
        id
    }
}

impl SimulationEngine {
    /// Captures all authoritative Stage 1 state required for exact continuation.
    #[must_use]
    pub fn export_snapshot(&self) -> EngineSnapshot {
        EngineSnapshot {
            world: self.state.clone(),
            pending_commands: self.pending_commands.clone(),
            executed_commands: self.executed_commands.clone(),
            events: self.event_ledger.records().to_vec(),
            next_command_id: self.next_command_id,
            next_event_id: self.event_ledger.next_event_id(),
        }
    }

    /// Creates a versioned manual or autosave bundle from authoritative state.
    ///
    /// # Errors
    ///
    /// Returns [`SaveError`] if internal invariants fail or serialization cannot
    /// produce a valid bundle.
    pub fn save_bundle(&self, kind: SaveKind) -> Result<SaveBundle, SaveError> {
        create_bundle(&self.export_snapshot(), kind)
    }

    /// Restores a new deterministic engine from a validated save bundle.
    ///
    /// # Errors
    ///
    /// Returns [`SaveError`] before constructing the engine if metadata,
    /// checksum, binary state, version or snapshot invariants are invalid.
    pub fn from_save_bundle(bundle: &SaveBundle) -> Result<Self, SaveError> {
        let (_, snapshot) = decode_bundle(bundle)?;
        Ok(Self {
            state: snapshot.world,
            pending_commands: snapshot.pending_commands,
            executed_commands: snapshot.executed_commands,
            event_ledger: EventLedger::from_snapshot(snapshot.events, snapshot.next_event_id),
            next_command_id: snapshot.next_command_id,
        })
    }

    /// Returns the canonical digest of every authoritative Stage 1 state field.
    ///
    /// Unlike the early minimal digest, this includes pending commands,
    /// executed-command history, Event Ledger contents and the next command and
    /// event identifiers. It hashes the same canonical binary representation
    /// used by save/restore, so persistence and determinism cannot silently
    /// disagree about what counts as authoritative state.
    ///
    /// # Errors
    ///
    /// Returns [`SaveError`] if authoritative state invariants fail.
    pub fn authoritative_state_digest(&self) -> Result<u64, SaveError> {
        let bundle = self.save_bundle(SaveKind::Manual)?;
        Ok(checksum_fnv1a64(&bundle.state_binary))
    }

    /// Returns every accepted command exactly once in original acceptance order.
    ///
    /// Executed and still-pending commands are merged by monotonic `CommandId`.
    /// The journal is sufficient to reconstruct command submissions from a
    /// fresh engine when combined with the original seed and target tick count.
    #[must_use]
    pub fn accepted_command_journal(&self) -> Vec<QueuedCommand> {
        let mut journal: Vec<_> = self
            .executed_commands
            .iter()
            .map(|record| record.command.clone())
            .chain(self.pending_commands.iter().cloned())
            .collect();
        journal.sort_by_key(|command| command.id);
        journal
    }

    /// Replays an accepted-command journal from a clean engine.
    ///
    /// `None` means the journal is not self-consistent for the supplied seed and
    /// target tick count. Valid project-generated journals reproduce the exact
    /// accepted-command IDs, submission dates, effective dates and resulting
    /// Event Ledger.
    #[must_use]
    pub fn replay_from_journal(
        seed: u64,
        target_elapsed_days: u64,
        journal: &[QueuedCommand],
    ) -> Option<Self> {
        let mut canonical = journal.to_vec();
        canonical.sort_by_key(|command| command.id);

        for (index, command) in canonical.iter().enumerate() {
            let expected_id = u64::try_from(index).ok()?.checked_add(1)?;
            if command.id.0 != expected_id {
                return None;
            }
        }

        let mut engine = Self::new(seed);
        let mut next_command = 0_usize;

        loop {
            while let Some(expected) = canonical.get(next_command) {
                if expected.submitted_on < engine.state.date {
                    return None;
                }
                if expected.submitted_on != engine.state.date {
                    break;
                }

                let assigned_id = engine
                    .submit_command(CommandRequest {
                        source: expected.source,
                        timing: CommandTiming::Scheduled(expected.execute_on),
                        payload: expected.payload.clone(),
                    })
                    .ok()?;

                if assigned_id != expected.id {
                    return None;
                }
                next_command += 1;
            }

            if engine.state.elapsed_days == target_elapsed_days {
                break;
            }
            if engine.state.elapsed_days > target_elapsed_days {
                return None;
            }
            engine.tick();
        }

        if next_command != canonical.len() {
            return None;
        }

        Some(engine)
    }

    /// Counter-based deterministic random sample for simulation-domain logic.
    ///
    /// The sample is a pure function of the world seed, current simulation tick,
    /// stable stream identifier and caller-provided ordinal. No mutable RNG
    /// cursor exists, so extra random calls in one subsystem cannot shift the
    /// random sequence used by another subsystem.
    #[must_use]
    pub fn deterministic_random_u64(&self, stream: u64, ordinal: u64) -> u64 {
        counter_random_u64(self.state.seed, self.state.elapsed_days, stream, ordinal)
    }
}

#[must_use]
fn counter_random_u64(seed: u64, tick: u64, stream: u64, ordinal: u64) -> u64 {
    let mut value = seed
        ^ tick.wrapping_mul(0x9e37_79b9_7f4a_7c15)
        ^ stream.rotate_left(23)
        ^ ordinal.wrapping_mul(0xd6e8_feb8_6659_fd93);

    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

#[cfg(test)]
mod tests {
    use simulation_model::{
        CommandId, CommandPayload, CommandTiming, CountryId, EventCategory, EventPayload,
        EventSource, SimulationDate,
    };
    use simulation_save::SaveKind;

    use super::{EventDraft, EventLedger};
    use crate::{SimulationClock, SimulationEngine, SimulationSpeed};
    use std::time::Duration;

    #[test]
    fn event_ids_are_monotonic_and_records_are_append_only() {
        let mut ledger = EventLedger::new();
        let first = ledger.append(
            SimulationDate::START,
            0,
            EventDraft {
                category: EventCategory::System,
                source: EventSource::System,
                payload: EventPayload::CommandExecuted {
                    command_id: CommandId(1),
                },
            },
        );
        let second = ledger.append(
            SimulationDate::new(1, 1, 2),
            1,
            EventDraft {
                category: EventCategory::System,
                source: EventSource::System,
                payload: EventPayload::CommandExecuted {
                    command_id: CommandId(2),
                },
            },
        );

        assert!(first < second);
        assert_eq!(ledger.records().len(), 2);
        assert_eq!(ledger.records()[0].id, first);
        assert_eq!(ledger.records()[1].id, second);
    }

    #[test]
    fn save_restore_preserves_pending_executed_and_event_state_exactly() {
        let mut engine = SimulationEngine::new(55);
        engine
            .submit_user_command(CommandTiming::Immediate, CommandPayload::NoOp { token: 1 })
            .expect("immediate command should be valid");
        engine
            .submit_country_ai_command(
                CountryId(7),
                CommandTiming::Scheduled(SimulationDate::new(1, 2, 1)),
                CommandPayload::NoOp { token: 2 },
            )
            .expect("scheduled command should be valid");
        engine.advance_days(10);

        let bundle = engine
            .save_bundle(SaveKind::Manual)
            .expect("engine should create a valid save");
        let restored =
            SimulationEngine::from_save_bundle(&bundle).expect("valid save should restore");

        assert_eq!(restored.export_snapshot(), engine.export_snapshot());
        assert_eq!(
            restored
                .authoritative_state_digest()
                .expect("restored digest should be valid"),
            engine
                .authoritative_state_digest()
                .expect("original digest should be valid")
        );
    }

    #[test]
    fn restored_engine_continues_identically_through_a_hundred_year_horizon() {
        let mut original = SimulationEngine::new(2026);
        original
            .submit_user_command(CommandTiming::Immediate, CommandPayload::NoOp { token: 10 })
            .expect("immediate command should be valid");
        original
            .submit_country_ai_command(
                CountryId(2),
                CommandTiming::Scheduled(SimulationDate::new(75, 1, 1)),
                CommandPayload::NoOp { token: 20 },
            )
            .expect("future command should be valid");

        original.advance_days(50 * 365);
        let bundle = original
            .save_bundle(SaveKind::Autosave)
            .expect("mid-run autosave should succeed");
        let mut restored =
            SimulationEngine::from_save_bundle(&bundle).expect("autosave should restore");

        assert_eq!(restored.export_snapshot(), original.export_snapshot());

        original.advance_days(50 * 365);
        restored.advance_days(50 * 365);

        assert_eq!(restored.export_snapshot(), original.export_snapshot());
        assert_eq!(
            restored
                .authoritative_state_digest()
                .expect("restored digest should be valid"),
            original
                .authoritative_state_digest()
                .expect("original digest should be valid")
        );
        assert_eq!(restored.command_log(), original.command_log());
        assert_eq!(restored.event_log(), original.event_log());
        assert_eq!(original.state().date, SimulationDate::new(101, 1, 1));
    }

    #[test]
    fn authoritative_digest_includes_pending_command_state() {
        let baseline = SimulationEngine::new(41);
        let mut with_pending = SimulationEngine::new(41);
        with_pending
            .submit_user_command(
                CommandTiming::Scheduled(SimulationDate::new(1, 2, 1)),
                CommandPayload::NoOp { token: 9 },
            )
            .expect("pending command should be valid");

        assert_ne!(
            baseline
                .authoritative_state_digest()
                .expect("baseline digest should be valid"),
            with_pending
                .authoritative_state_digest()
                .expect("pending digest should be valid")
        );
    }

    #[test]
    fn counter_random_is_repeatable_and_call_order_independent() {
        let engine = SimulationEngine::new(0x1234_5678_9abc_def0);
        let first_a = engine.deterministic_random_u64(11, 0);
        let first_b = engine.deterministic_random_u64(22, 0);
        let second_b = engine.deterministic_random_u64(22, 0);
        let second_a = engine.deterministic_random_u64(11, 0);

        assert_eq!(first_a, second_a);
        assert_eq!(first_b, second_b);
        assert_ne!(first_a, first_b);
        assert_eq!(engine.state().elapsed_days, 0);
    }

    #[test]
    fn counter_random_changes_with_seed_tick_stream_and_ordinal() {
        let mut base = SimulationEngine::new(7);
        let seed_value = base.deterministic_random_u64(3, 4);
        let different_stream = base.deterministic_random_u64(4, 4);
        let different_ordinal = base.deterministic_random_u64(3, 5);

        let other_seed = SimulationEngine::new(8).deterministic_random_u64(3, 4);
        base.step_one_day();
        let different_tick = base.deterministic_random_u64(3, 4);

        assert_ne!(seed_value, different_stream);
        assert_ne!(seed_value, different_ordinal);
        assert_ne!(seed_value, other_seed);
        assert_ne!(seed_value, different_tick);
    }

    #[test]
    fn accepted_command_journal_replays_exact_authoritative_state() {
        let mut original = SimulationEngine::new(88);
        original
            .submit_user_command(CommandTiming::Immediate, CommandPayload::NoOp { token: 1 })
            .expect("initial command should be valid");
        original.advance_days(3);
        original
            .submit_country_ai_command(
                CountryId(5),
                CommandTiming::Scheduled(SimulationDate::new(1, 1, 20)),
                CommandPayload::NoOp { token: 2 },
            )
            .expect("AI command should be valid");
        original.advance_days(37);
        original
            .submit_user_command(
                CommandTiming::Scheduled(SimulationDate::new(1, 3, 1)),
                CommandPayload::NoOp { token: 3 },
            )
            .expect("final-date pending command should be valid");

        let journal = original.accepted_command_journal();
        let replayed = SimulationEngine::replay_from_journal(
            original.state().seed,
            original.state().elapsed_days,
            &journal,
        )
        .expect("project-generated journal should replay");

        assert_eq!(replayed.export_snapshot(), original.export_snapshot());
        assert_eq!(
            replayed
                .authoritative_state_digest()
                .expect("replayed digest should be valid"),
            original
                .authoritative_state_digest()
                .expect("original digest should be valid")
        );
    }

    #[test]
    fn ten_thousand_tick_replay_is_exact() {
        let mut original = SimulationEngine::new(991);
        original
            .submit_user_command(CommandTiming::Immediate, CommandPayload::NoOp { token: 10 })
            .expect("initial command should be valid");
        original.advance_days(500);
        original
            .submit_country_ai_command(
                CountryId(9),
                CommandTiming::Scheduled(SimulationDate::new(20, 1, 1)),
                CommandPayload::NoOp { token: 20 },
            )
            .expect("future AI command should be valid");
        original.advance_days(9_500);

        let journal = original.accepted_command_journal();
        let replayed = SimulationEngine::replay_from_journal(991, 10_000, &journal)
            .expect("10k-tick journal should replay");

        assert_eq!(replayed.export_snapshot(), original.export_snapshot());
        assert_eq!(
            replayed
                .authoritative_state_digest()
                .expect("replayed digest should be valid"),
            original
                .authoritative_state_digest()
                .expect("original digest should be valid")
        );
    }

    #[test]
    fn manual_and_high_speed_playback_have_same_authoritative_digest() {
        fn seed_commands(engine: &mut SimulationEngine) {
            engine
                .submit_user_command(CommandTiming::Immediate, CommandPayload::NoOp { token: 1 })
                .expect("user command should be valid");
            engine
                .submit_country_ai_command(
                    CountryId(4),
                    CommandTiming::Scheduled(SimulationDate::new(1, 6, 1)),
                    CommandPayload::NoOp { token: 2 },
                )
                .expect("AI command should be valid");
        }

        let mut manual = SimulationEngine::new(101);
        seed_commands(&mut manual);
        manual.advance_days(365);

        let mut realtime = SimulationEngine::new(101);
        seed_commands(&mut realtime);
        let mut clock = SimulationClock::new(SimulationSpeed::X365);
        clock.advance_real_time(&mut realtime, Duration::from_secs(1));

        assert_eq!(realtime.export_snapshot(), manual.export_snapshot());
        assert_eq!(
            realtime
                .authoritative_state_digest()
                .expect("realtime digest should be valid"),
            manual
                .authoritative_state_digest()
                .expect("manual digest should be valid")
        );
    }
}
