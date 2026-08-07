#![forbid(unsafe_code)]

mod event;
mod terrain;
mod terrain_effects;
mod world;

pub use event::{EventCategory, EventFilter, EventId, EventPayload, EventRecord, EventSource};
pub use terrain::{
    trial_terrain_bounds, BiomeClass, ReliefClass, TerrainError, TerrainHydrology,
    TerrainLandmasses, TerrainSample, TerrainState, NO_DOWNSTREAM_INDEX, TERRAIN_GRID_SIDE,
    TERRAIN_GRID_SPACING_M, TERRAIN_SAMPLE_COUNT, TERRAIN_SEA_LEVEL_M, TRIAL_WORLD_HALF_EXTENT_M,
};
pub use terrain_effects::{derive_terrain_effects, TerrainEffectField, TerrainEffects};
pub use world::{
    MapPoint, RegionPoliticalState, RegionState, RegionSurface, WorldBounds, WorldSpatialError,
    WorldSpatialState, TRIAL_REGION_COUNT,
};

/// Stable identifier for a country in the trial simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CountryId(pub u16);

/// Stable identifier for a region in the trial simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RegionId(pub u16);

/// Calendar boundary information produced after one simulated day advances.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DateBoundary {
    pub month_changed: bool,
    pub quarter_changed: bool,
    pub year_changed: bool,
}

/// Proleptic simulation date. The trial begins at Year 1, Month 1, Day 1.
///
/// The trial deliberately uses a fixed 365-day calendar without leap years so
/// that date progression is simple, deterministic and independent of host
/// locale or wall-clock time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SimulationDate {
    pub year: u32,
    pub month: u8,
    pub day: u8,
}

impl SimulationDate {
    pub const START: Self = Self {
        year: 1,
        month: 1,
        day: 1,
    };

    const MONTH_LENGTHS: [u8; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

    #[must_use]
    pub const fn new(year: u32, month: u8, day: u8) -> Self {
        Self { year, month, day }
    }

    #[must_use]
    pub const fn is_valid(self) -> bool {
        if self.year == 0 || self.month == 0 || self.month > 12 || self.day == 0 {
            return false;
        }
        self.day <= Self::MONTH_LENGTHS[(self.month - 1) as usize]
    }

    #[must_use]
    pub fn month_length(self) -> u8 {
        Self::MONTH_LENGTHS[(self.month.saturating_sub(1).min(11)) as usize]
    }

    /// Quarter number in the range 1..=4.
    #[must_use]
    pub fn quarter(self) -> u8 {
        ((self.month.saturating_sub(1).min(11)) / 3) + 1
    }

    /// Ordinal day in the range 1..=365 for valid dates.
    #[must_use]
    pub fn day_of_year(self) -> u16 {
        let completed_months = Self::MONTH_LENGTHS
            .iter()
            .take(usize::from(self.month.saturating_sub(1).min(11)))
            .map(|&days| u16::from(days))
            .sum::<u16>();
        completed_months + u16::from(self.day)
    }

    /// Advances exactly one simulated day and reports calendar boundaries.
    pub fn advance_one_day(&mut self) -> DateBoundary {
        let previous_month = self.month;
        let previous_quarter = self.quarter();
        let previous_year = self.year;
        let month_length = self.month_length();

        if self.day < month_length {
            self.day += 1;
        } else if self.month < 12 {
            self.month += 1;
            self.day = 1;
        } else {
            self.year += 1;
            self.month = 1;
            self.day = 1;
        }

        DateBoundary {
            month_changed: self.month != previous_month,
            quarter_changed: self.quarter() != previous_quarter,
            year_changed: self.year != previous_year,
        }
    }
}

/// Monotonic identifier assigned by the simulation core when a command is
/// accepted. It is also the final deterministic tie-breaker for same-day,
/// same-priority commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CommandId(pub u64);

/// Origin of an accepted simulation command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandSource {
    User,
    CountryAi(CountryId),
}

/// When an accepted command becomes eligible for execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandTiming {
    /// Effective on the simulation's current date and processed at the start of
    /// the next deterministic daily tick.
    Immediate,
    /// Effective on the specified simulation date.
    Scheduled(SimulationDate),
}

/// Fixed Stage 1 command lanes. Lower values execute first.
///
/// User intervention precedes autonomous country AI on the same date. Within
/// the same lane, `CommandId` preserves submission order deterministically.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum CommandPriority {
    UserIntervention = 10,
    CountryAi = 20,
}

impl CommandPriority {
    #[must_use]
    pub const fn for_source(source: CommandSource) -> Self {
        match source {
            CommandSource::User => Self::UserIntervention,
            CommandSource::CountryAi(_) => Self::CountryAi,
        }
    }
}

/// Extensible payload carried by the Stage 1 command infrastructure.
///
/// Domain commands for policy, diplomacy and military control are deliberately
/// not invented in Stage 1. `NoOp` is retained as an infrastructure health
/// probe so scheduling, ordering, logging and replay can be validated before
/// those domain systems exist.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CommandPayload {
    NoOp { token: u64 },
}

/// Command request before the core assigns identity, resolved date and
/// priority.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CommandRequest {
    pub source: CommandSource,
    pub timing: CommandTiming,
    pub payload: CommandPayload,
}

impl CommandRequest {
    #[must_use]
    pub const fn user(timing: CommandTiming, payload: CommandPayload) -> Self {
        Self {
            source: CommandSource::User,
            timing,
            payload,
        }
    }

    #[must_use]
    pub const fn country_ai(
        country_id: CountryId,
        timing: CommandTiming,
        payload: CommandPayload,
    ) -> Self {
        Self {
            source: CommandSource::CountryAi(country_id),
            timing,
            payload,
        }
    }
}

/// Canonical accepted command stored in the deterministic queue.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QueuedCommand {
    pub id: CommandId,
    pub source: CommandSource,
    pub submitted_on: SimulationDate,
    pub execute_on: SimulationDate,
    pub priority: CommandPriority,
    pub payload: CommandPayload,
}

/// Append-only record produced when a queued command is executed.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CommandExecutionRecord {
    pub command: QueuedCommand,
    pub executed_on: SimulationDate,
}

/// Rejection returned synchronously when a command request is invalid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandError {
    InvalidScheduledDate {
        requested: SimulationDate,
    },
    ScheduledInPast {
        requested: SimulationDate,
        current: SimulationDate,
    },
    CommandIdExhausted,
}

impl core::fmt::Display for CommandError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidScheduledDate { requested } => write!(
                formatter,
                "invalid scheduled simulation date {:04}-{:02}-{:02}",
                requested.year, requested.month, requested.day
            ),
            Self::ScheduledInPast { requested, current } => write!(
                formatter,
                "scheduled date {:04}-{:02}-{:02} is before current date {:04}-{:02}-{:02}",
                requested.year,
                requested.month,
                requested.day,
                current.year,
                current.month,
                current.day
            ),
            Self::CommandIdExhausted => formatter.write_str("command identifier space exhausted"),
        }
    }
}

impl std::error::Error for CommandError {}

/// Authoritative world state owned only by the simulation core.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldState {
    pub date: SimulationDate,
    pub elapsed_days: u64,
    pub seed: u64,
    pub terrain: Option<TerrainState>,
    pub spatial: WorldSpatialState,
}

impl WorldState {
    #[must_use]
    pub const fn new(seed: u64) -> Self {
        Self {
            date: SimulationDate::START,
            elapsed_days: 0,
            seed,
            terrain: None,
            spatial: WorldSpatialState::uninitialized(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CommandPayload, CommandPriority, CommandRequest, CommandSource, CommandTiming, CountryId,
        DateBoundary, SimulationDate, WorldState,
    };

    #[test]
    fn start_state_is_stable() {
        let world = WorldState::new(42);
        assert_eq!(world.date, SimulationDate::START);
        assert_eq!(world.elapsed_days, 0);
        assert_eq!(world.seed, 42);
        assert!(world.terrain.is_none());
        assert!(!world.spatial.is_initialized());
        assert!(world.spatial.regions.is_empty());
    }

    #[test]
    fn known_dates_validate() {
        assert!(SimulationDate::START.is_valid());
        assert!(SimulationDate::new(1, 2, 28).is_valid());
        assert!(!SimulationDate::new(0, 1, 1).is_valid());
        assert!(!SimulationDate::new(1, 2, 29).is_valid());
        assert!(!SimulationDate::new(1, 13, 1).is_valid());
    }

    #[test]
    fn calendar_rolls_over_month_quarter_and_year() {
        let mut month = SimulationDate::new(1, 1, 31);
        assert_eq!(
            month.advance_one_day(),
            DateBoundary {
                month_changed: true,
                quarter_changed: false,
                year_changed: false,
            }
        );
        assert_eq!(month, SimulationDate::new(1, 2, 1));

        let mut quarter = SimulationDate::new(1, 3, 31);
        assert!(quarter.advance_one_day().quarter_changed);
        assert_eq!(quarter, SimulationDate::new(1, 4, 1));

        let mut year = SimulationDate::new(1, 12, 31);
        let boundary = year.advance_one_day();
        assert!(boundary.month_changed);
        assert!(boundary.quarter_changed);
        assert!(boundary.year_changed);
        assert_eq!(year, SimulationDate::new(2, 1, 1));
    }

    #[test]
    fn ordinal_day_and_quarter_are_stable() {
        assert_eq!(SimulationDate::new(1, 1, 1).day_of_year(), 1);
        assert_eq!(SimulationDate::new(1, 12, 31).day_of_year(), 365);
        assert_eq!(SimulationDate::new(1, 1, 1).quarter(), 1);
        assert_eq!(SimulationDate::new(1, 4, 1).quarter(), 2);
        assert_eq!(SimulationDate::new(1, 7, 1).quarter(), 3);
    }

    #[test]
    fn command_constructors_preserve_origin_and_default_priority_lane() {
        let user =
            CommandRequest::user(CommandTiming::Immediate, CommandPayload::NoOp { token: 1 });
        let ai = CommandRequest::country_ai(
            CountryId(3),
            CommandTiming::Immediate,
            CommandPayload::NoOp { token: 2 },
        );

        assert_eq!(user.source, CommandSource::User);
        assert_eq!(ai.source, CommandSource::CountryAi(CountryId(3)));
        assert!(CommandPriority::for_source(user.source) < CommandPriority::for_source(ai.source));
    }
}
