use crate::{CommandId, CountryId, SimulationDate};

/// Monotonic identifier assigned to an immutable world event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EventId(pub u64);

/// Stable top-level event categories used by the Event Ledger and UI filters.
///
/// Categories exist before their full domain systems so later stages can emit
/// typed facts without redesigning the ledger storage contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EventCategory {
    System,
    Economic,
    Policy,
    Diplomatic,
    Military,
    War,
    Occupation,
    Treaty,
    UserIntervention,
}

/// Origin that caused or authored an event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventSource {
    System,
    User,
    Country(CountryId),
}

/// Typed facts currently emitted by the Stage 1 core.
///
/// Later domain stages extend this enum with economic, diplomatic, military,
/// war, occupation and treaty facts. Stage 1 deliberately records only facts
/// that truly exist at this point in development.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventPayload {
    CommandExecuted { command_id: CommandId },
}

/// Immutable fact stored in the append-only Event Ledger.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EventRecord {
    pub id: EventId,
    pub occurred_on: SimulationDate,
    /// Zero-based daily tick position at which the event occurred.
    pub tick_index: u64,
    pub category: EventCategory,
    pub source: EventSource,
    pub payload: EventPayload,
}

/// Query filter used by the presentation and analysis layers.
///
/// Every field is optional. Populated fields are combined with logical AND.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EventFilter {
    pub category: Option<EventCategory>,
    pub source: Option<EventSource>,
    pub from_date: Option<SimulationDate>,
    pub through_date: Option<SimulationDate>,
}

impl EventFilter {
    #[must_use]
    pub fn matches(self, event: &EventRecord) -> bool {
        if let Some(category) = self.category
            && event.category != category
        {
            return false;
        }

        if let Some(source) = self.source
            && event.source != source
        {
            return false;
        }

        if let Some(from_date) = self.from_date
            && event.occurred_on < from_date
        {
            return false;
        }

        if let Some(through_date) = self.through_date
            && event.occurred_on > through_date
        {
            return false;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::{EventCategory, EventFilter, EventId, EventPayload, EventRecord, EventSource};
    use crate::{CommandId, CountryId, SimulationDate};

    fn sample_event() -> EventRecord {
        EventRecord {
            id: EventId(7),
            occurred_on: SimulationDate::new(1, 2, 3),
            tick_index: 33,
            category: EventCategory::Policy,
            source: EventSource::Country(CountryId(4)),
            payload: EventPayload::CommandExecuted {
                command_id: CommandId(9),
            },
        }
    }

    #[test]
    fn empty_filter_matches_every_event() {
        assert!(EventFilter::default().matches(&sample_event()));
    }

    #[test]
    fn filter_combines_fields_with_logical_and() {
        let filter = EventFilter {
            category: Some(EventCategory::Policy),
            source: Some(EventSource::Country(CountryId(4))),
            from_date: Some(SimulationDate::new(1, 2, 1)),
            through_date: Some(SimulationDate::new(1, 2, 28)),
        };
        assert!(filter.matches(&sample_event()));

        let wrong_country = EventFilter {
            source: Some(EventSource::Country(CountryId(5))),
            ..filter
        };
        assert!(!wrong_country.matches(&sample_event()));
    }
}
