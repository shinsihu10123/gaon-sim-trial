use simulation_model::{EventCategory, EventId, EventPayload, EventRecord, EventSource, SimulationDate};

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

    pub(crate) fn records(&self) -> &[EventRecord] {
        &self.records
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

#[cfg(test)]
mod tests {
    use simulation_model::{CommandId, EventCategory, EventPayload, EventSource, SimulationDate};

    use super::{EventDraft, EventLedger};

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
}
