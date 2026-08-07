#![forbid(unsafe_code)]

use simulation_model::{SimulationDate, WorldState};

pub const SAVE_FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveMetadata {
    pub format_version: u32,
    pub date: SimulationDate,
    pub elapsed_days: u64,
    pub seed: u64,
}

impl From<&WorldState> for SaveMetadata {
    fn from(state: &WorldState) -> Self {
        Self {
            format_version: SAVE_FORMAT_VERSION,
            date: state.date,
            elapsed_days: state.elapsed_days,
            seed: state.seed,
        }
    }
}

impl SaveMetadata {
    #[must_use]
    pub fn encode_text(&self) -> String {
        format!(
            "version={}\nyear={}\nmonth={}\nday={}\nelapsed_days={}\nseed={}\n",
            self.format_version,
            self.date.year,
            self.date.month,
            self.date.day,
            self.elapsed_days,
            self.seed
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{SaveMetadata, SAVE_FORMAT_VERSION};
    use simulation_model::WorldState;

    #[test]
    fn metadata_is_derived_from_world_state() {
        let state = WorldState::new(123);
        let metadata = SaveMetadata::from(&state);
        assert_eq!(metadata.format_version, SAVE_FORMAT_VERSION);
        assert!(metadata.encode_text().contains("seed=123"));
    }
}
