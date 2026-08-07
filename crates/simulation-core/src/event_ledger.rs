use simulation_model::{
    EventCategory, EventId, EventPayload, EventRecord, EventSource, SimulationDate,
};
use simulation_save::{
    create_bundle, decode_bundle, EngineSnapshot, SaveBundle, SaveError, SaveKind,
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
}

#[cfg(test)]
mod tests {
    use simulation_model::{
        CommandId, CommandPayload, CommandTiming, CountryId, EventCategory, EventPayload,
        EventSource, SimulationDate,
    };
    use simulation_save::SaveKind;

    use super::{EventDraft, EventLedger};
    use crate::SimulationEngine;

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
        assert_eq!(restored.state_digest(), original.state_digest());
        assert_eq!(restored.command_log(), original.command_log());
        assert_eq!(restored.event_log(), original.event_log());
        assert_eq!(original.state().date, SimulationDate::new(101, 1, 1));
    }
}
