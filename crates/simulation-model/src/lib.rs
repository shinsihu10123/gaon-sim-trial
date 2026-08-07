#![forbid(unsafe_code)]

mod event;
mod terrain;
mod world;

pub use event::{EventCategory, EventFilter, EventId, EventPayload, EventRecord, EventSource};
pub use terrain::{
    trial_terrain_bounds, BiomeClass, ReliefClass, TerrainError, TerrainHydrology,
    TerrainLandmasses, TerrainSample, TerrainState, NO_DOWNSTREAM_INDEX, TERRAIN_GRID_SIDE,
    TERRAIN_GRID_SPACING_M, TERRAIN_SAMPLE_COUNT, TERRAIN_SEA_LEVEL_M,
    TRIAL_WORLD_HALF_EXTENT_M,
};
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
    pub fn ordinal_day(self) -> u16 {
        let completed_months: u16 = Self::MONTH_LENGTHS
            .iter()
            .take(self.month.saturating_sub(1) as usize)
            .map(|&days| u16::from(days))
            .sum();
        completed_months + u16::from(self.day)
    }

    /// Advances exactly one simulated day and reports crossed calendar boundaries.
    #[must_use]
    pub fn next_day(self) -> (Self, DateBoundary) {
        let mut next = self;
        let previous_month = self.month;
        let previous_quarter = self.quarter();
        let previous_year = self.year;

        if next.day < next.month_length() {
            next.day += 1;
        } else {
            next.day = 1;
            if next.month < 12 {
                next.month += 1;
            } else {
                next.month = 1;
                next.year = next.year.saturating_add(1);
            }
        }

        (
            next,
            DateBoundary {
                month_changed: next.month != previous_month,
                quarter_changed: next.quarter() != previous_quarter,
                year_changed: next.year != previous_year,
            },
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CommandId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandSource {
    User,
    CountryAi { country: CountryId },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CommandPriority(pub u16);

impl CommandPriority {
    pub const USER: Self = Self(100);
    pub const COUNTRY_AI: Self = Self(200);
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CommandPayload {
    Marker { code: u32, value: i64 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueuedCommand {
    pub id: CommandId,
    pub source: CommandSource,
    pub submitted_on: SimulationDate,
    pub effective_on: SimulationDate,
    pub priority: CommandPriority,
    pub payload: CommandPayload,
}

impl QueuedCommand {
    #[must_use]
    pub fn user(
        id: CommandId,
        submitted_on: SimulationDate,
        effective_on: SimulationDate,
        payload: CommandPayload,
    ) -> Self {
        Self {
            id,
            source: CommandSource::User,
            submitted_on,
            effective_on,
            priority: CommandPriority::USER,
            payload,
        }
    }

    #[must_use]
    pub fn country_ai(
        id: CommandId,
        country: CountryId,
        submitted_on: SimulationDate,
        effective_on: SimulationDate,
        payload: CommandPayload,
    ) -> Self {
        Self {
            id,
            source: CommandSource::CountryAi { country },
            submitted_on,
            effective_on,
            priority: CommandPriority::COUNTRY_AI,
            payload,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandExecutionRecord {
    pub command_id: CommandId,
    pub executed_on: SimulationDate,
    pub source: CommandSource,
    pub payload: CommandPayload,
}

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
    pub fn new(seed: u64) -> Self {
        Self {
            date: SimulationDate::START,
            elapsed_days: 0,
            seed,
            terrain: None,
            spatial: WorldSpatialState::uninitialized(),
        }
    }

    #[must_use]
    pub fn tick_one_day(&mut self) -> DateBoundary {
        let (next_date, boundary) = self.date.next_day();
        self.date = next_date;
        self.elapsed_days = self.elapsed_days.saturating_add(1);
        boundary
    }
}

#[cfg(test)]
mod tests {
    use super::{CommandId, CommandPayload, CountryId, QueuedCommand, SimulationDate, WorldState};

    #[test]
    fn start_state_is_stable() {
        let world = WorldState::new(42);
        assert_eq!(world.date, SimulationDate::START);
        assert_eq!(world.elapsed_days, 0);
        assert_eq!(world.seed, 42);
        assert!(world.terrain.is_none());
        assert!(!world.spatial.is_initialized());
    }

    #[test]
    fn known_dates_validate() {
        assert!(SimulationDate::new(1, 1, 1).is_valid());
        assert!(SimulationDate::new(1, 2, 28).is_valid());
        assert!(!SimulationDate::new(1, 2, 29).is_valid());
        assert!(!SimulationDate::new(0, 1, 1).is_valid());
    }

    #[test]
    fn calendar_rolls_over_month_quarter_and_year() {
        let (february, month_boundary) = SimulationDate::new(1, 1, 31).next_day();
        assert_eq!(february, SimulationDate::new(1, 2, 1));
        assert!(month_boundary.month_changed);
        assert!(!month_boundary.quarter_changed);

        let (april, quarter_boundary) = SimulationDate::new(1, 3, 31).next_day();
        assert_eq!(april, SimulationDate::new(1, 4, 1));
        assert!(quarter_boundary.month_changed);
        assert!(quarter_boundary.quarter_changed);

        let (new_year, year_boundary) = SimulationDate::new(1, 12, 31).next_day();
        assert_eq!(new_year, SimulationDate::new(2, 1, 1));
        assert!(year_boundary.year_changed);
    }

    #[test]
    fn ordinal_day_and_quarter_are_stable() {
        assert_eq!(SimulationDate::new(1, 1, 1).ordinal_day(), 1);
        assert_eq!(SimulationDate::new(1, 12, 31).ordinal_day(), 365);
        assert_eq!(SimulationDate::new(1, 7, 1).quarter(), 3);
    }

    #[test]
    fn command_constructors_preserve_origin_and_default_priority_lane() {
        let date = SimulationDate::START;
        let user = QueuedCommand::user(
            CommandId(1),
            date,
            date,
            CommandPayload::Marker { code: 1, value: 10 },
        );
        let ai = QueuedCommand::country_ai(
            CommandId(2),
            CountryId(4),
            date,
            date,
            CommandPayload::Marker { code: 2, value: 20 },
        );
        assert!(user.priority < ai.priority);
    }
}
