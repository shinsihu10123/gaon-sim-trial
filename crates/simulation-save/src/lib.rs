#![forbid(unsafe_code)]

mod binary;

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use simulation_model::{
    CommandExecutionRecord, CommandId, CommandPriority, CommandSource, EventCategory, EventId,
    EventPayload, EventRecord, EventSource, QueuedCommand, SimulationDate, WorldState,
};

pub const SAVE_FORMAT_VERSION: u32 = 1;

/// Identifies why a snapshot was persisted. Manual and autosave snapshots use
/// the exact same state format and validation path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SaveKind {
    Manual,
    Autosave,
}

/// Human-readable JSON metadata stored beside the canonical binary state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveMetadata {
    pub format_version: u32,
    pub save_kind: SaveKind,
    pub simulation_year: u32,
    pub simulation_month: u8,
    pub simulation_day: u8,
    pub elapsed_days: u64,
    /// Hex encoding avoids JavaScript number precision loss in the future UI.
    pub seed_hex: String,
    pub state_bytes: u64,
    /// FNV-1a integrity checksum. This detects accidental corruption; it is not
    /// intended as a cryptographic authenticity mechanism.
    pub state_checksum_fnv1a64: String,
}

/// Complete authoritative Stage 1 simulation state required for exact resume.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineSnapshot {
    pub world: WorldState,
    pub pending_commands: Vec<QueuedCommand>,
    pub executed_commands: Vec<CommandExecutionRecord>,
    pub events: Vec<EventRecord>,
    pub next_command_id: u64,
    pub next_event_id: u64,
}

/// Portable save payload. A browser adapter can persist the JSON and binary
/// members separately without changing the deterministic core format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveBundle {
    pub metadata_json: String,
    pub state_binary: Vec<u8>,
}

impl SaveBundle {
    /// Parses metadata without decoding the binary engine state.
    ///
    /// # Errors
    ///
    /// Returns [`SaveError::MetadataDecode`] for malformed JSON.
    pub fn metadata(&self) -> Result<SaveMetadata, SaveError> {
        serde_json::from_str(&self.metadata_json)
            .map_err(|error| SaveError::MetadataDecode(error.to_string()))
    }
}

/// Deterministic autosave cadence helper. The caller owns the last successful
/// autosave day so failed writes never advance the schedule accidentally.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AutosavePolicy {
    interval_days: u64,
}

impl AutosavePolicy {
    /// Creates an autosave policy measured in simulated days.
    ///
    /// # Errors
    ///
    /// Returns [`SaveError::InvalidAutosaveInterval`] when `interval_days` is
    /// zero.
    pub const fn new(interval_days: u64) -> Result<Self, SaveError> {
        if interval_days == 0 {
            return Err(SaveError::InvalidAutosaveInterval);
        }
        Ok(Self { interval_days })
    }

    #[must_use]
    pub const fn interval_days(self) -> u64 {
        self.interval_days
    }

    #[must_use]
    pub fn is_due(self, current_elapsed_days: u64, last_saved_day: Option<u64>) -> bool {
        match last_saved_day {
            Some(last_saved_day) => {
                current_elapsed_days.saturating_sub(last_saved_day) >= self.interval_days
            }
            None => current_elapsed_days >= self.interval_days,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveError {
    MetadataEncode(String),
    MetadataDecode(String),
    UnsupportedMetadataVersion { found: u32 },
    UnsupportedBinaryVersion { found: u32 },
    InvalidMetadata(&'static str),
    InvalidBinary(&'static str),
    ChecksumMismatch { expected: u64, actual: u64 },
    TruncatedBinary,
    TrailingBinaryData,
    InvalidTag { field: &'static str, tag: u8 },
    RecordCountOverflow { section: &'static str },
    RecordLimitExceeded { section: &'static str, count: u64 },
    SnapshotInvariant(&'static str),
    InvalidAutosaveInterval,
}

impl core::fmt::Display for SaveError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::MetadataEncode(message) => write!(formatter, "metadata encode failed: {message}"),
            Self::MetadataDecode(message) => write!(formatter, "metadata decode failed: {message}"),
            Self::UnsupportedMetadataVersion { found } => {
                write!(formatter, "unsupported metadata save version {found}")
            }
            Self::UnsupportedBinaryVersion { found } => {
                write!(formatter, "unsupported binary save version {found}")
            }
            Self::InvalidMetadata(message) => write!(formatter, "invalid save metadata: {message}"),
            Self::InvalidBinary(message) => write!(formatter, "invalid binary state: {message}"),
            Self::ChecksumMismatch { expected, actual } => write!(
                formatter,
                "binary state checksum mismatch: expected {expected:016x}, got {actual:016x}"
            ),
            Self::TruncatedBinary => formatter.write_str("binary state is truncated"),
            Self::TrailingBinaryData => formatter.write_str("binary state contains trailing data"),
            Self::InvalidTag { field, tag } => {
                write!(formatter, "invalid {field} tag {tag}")
            }
            Self::RecordCountOverflow { section } => {
                write!(
                    formatter,
                    "{section} count cannot be represented by save format"
                )
            }
            Self::RecordLimitExceeded { section, count } => {
                write!(
                    formatter,
                    "{section} record count {count} exceeds safety limit"
                )
            }
            Self::SnapshotInvariant(message) => {
                write!(formatter, "snapshot invariant failed: {message}")
            }
            Self::InvalidAutosaveInterval => {
                formatter.write_str("autosave interval must be at least one simulated day")
            }
        }
    }
}

impl std::error::Error for SaveError {}

/// Encodes a validated authoritative engine snapshot into JSON metadata and
/// canonical binary state.
///
/// # Errors
///
/// Returns [`SaveError`] when snapshot invariants fail, counts cannot be
/// represented, or metadata serialization fails.
pub fn create_bundle(snapshot: &EngineSnapshot, kind: SaveKind) -> Result<SaveBundle, SaveError> {
    validate_snapshot(snapshot)?;
    let state_binary = binary::encode_snapshot(snapshot)?;
    let state_bytes = u64::try_from(state_binary.len())
        .map_err(|_| SaveError::InvalidMetadata("state byte length overflow"))?;
    let checksum = checksum_fnv1a64(&state_binary);

    let metadata = SaveMetadata {
        format_version: SAVE_FORMAT_VERSION,
        save_kind: kind,
        simulation_year: snapshot.world.date.year,
        simulation_month: snapshot.world.date.month,
        simulation_day: snapshot.world.date.day,
        elapsed_days: snapshot.world.elapsed_days,
        seed_hex: format!("{:016x}", snapshot.world.seed),
        state_bytes,
        state_checksum_fnv1a64: format!("{checksum:016x}"),
    };

    let metadata_json = serde_json::to_string_pretty(&metadata)
        .map_err(|error| SaveError::MetadataEncode(error.to_string()))?;

    Ok(SaveBundle {
        metadata_json,
        state_binary,
    })
}

/// Validates and decodes a save bundle into an authoritative engine snapshot.
///
/// # Errors
///
/// Returns [`SaveError`] for malformed metadata, unsupported versions,
/// corruption, structural violations, or metadata/state disagreement.
pub fn decode_bundle(bundle: &SaveBundle) -> Result<(SaveKind, EngineSnapshot), SaveError> {
    let metadata = bundle.metadata()?;
    if metadata.format_version != SAVE_FORMAT_VERSION {
        return Err(SaveError::UnsupportedMetadataVersion {
            found: metadata.format_version,
        });
    }

    let actual_length = u64::try_from(bundle.state_binary.len())
        .map_err(|_| SaveError::InvalidMetadata("state byte length overflow"))?;
    if metadata.state_bytes != actual_length {
        return Err(SaveError::InvalidMetadata(
            "metadata state length does not match binary payload",
        ));
    }

    let expected_checksum = parse_hex_u64(
        &metadata.state_checksum_fnv1a64,
        "invalid state checksum encoding",
    )?;
    let actual_checksum = checksum_fnv1a64(&bundle.state_binary);
    if actual_checksum != expected_checksum {
        return Err(SaveError::ChecksumMismatch {
            expected: expected_checksum,
            actual: actual_checksum,
        });
    }

    let snapshot = binary::decode_snapshot(&bundle.state_binary)?;
    validate_snapshot(&snapshot)?;
    validate_metadata_matches_snapshot(&metadata, &snapshot)?;

    Ok((metadata.save_kind, snapshot))
}

/// Verifies all cross-record invariants required for exact deterministic resume.
///
/// # Errors
///
/// Returns [`SaveError::SnapshotInvariant`] when authoritative state is
/// internally inconsistent.
pub fn validate_snapshot(snapshot: &EngineSnapshot) -> Result<(), SaveError> {
    validate_world(&snapshot.world)?;

    let mut command_ids = BTreeSet::new();
    let mut executed_sources = BTreeMap::new();

    for command in &snapshot.pending_commands {
        validate_command(command, &snapshot.world)?;
        if command.execute_on < snapshot.world.date {
            return Err(SaveError::SnapshotInvariant(
                "pending command execution date is already in the past",
            ));
        }
        if !command_ids.insert(command.id) {
            return Err(SaveError::SnapshotInvariant("duplicate command id"));
        }
    }

    if !snapshot
        .pending_commands
        .windows(2)
        .all(|pair| command_key(&pair[0]) <= command_key(&pair[1]))
    {
        return Err(SaveError::SnapshotInvariant(
            "pending command queue is not in canonical order",
        ));
    }

    for record in &snapshot.executed_commands {
        validate_execution_record(record, &snapshot.world)?;
        if !command_ids.insert(record.command.id) {
            return Err(SaveError::SnapshotInvariant("duplicate command id"));
        }
        executed_sources.insert(record.command.id, record.command.source);
    }

    if !snapshot
        .executed_commands
        .windows(2)
        .all(|pair| execution_key(&pair[0]) <= execution_key(&pair[1]))
    {
        return Err(SaveError::SnapshotInvariant(
            "executed command log is not in canonical order",
        ));
    }

    let expected_next_command = command_ids.last().map_or(1, |id| id.0.saturating_add(1));
    if snapshot.next_command_id != expected_next_command || snapshot.next_command_id == 0 {
        return Err(SaveError::SnapshotInvariant(
            "next command id does not follow accepted command history",
        ));
    }

    validate_events(snapshot, &executed_sources)?;
    Ok(())
}

#[must_use]
pub fn checksum_fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for &byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

fn validate_world(world: &WorldState) -> Result<(), SaveError> {
    if !world.date.is_valid() {
        return Err(SaveError::SnapshotInvariant("world date is invalid"));
    }

    let expected_elapsed = u64::from(world.date.year - 1)
        .saturating_mul(365)
        .saturating_add(u64::from(world.date.day_of_year() - 1));
    if world.elapsed_days != expected_elapsed {
        return Err(SaveError::SnapshotInvariant(
            "world date and elapsed day counter disagree",
        ));
    }
    Ok(())
}

fn validate_command(command: &QueuedCommand, world: &WorldState) -> Result<(), SaveError> {
    if command.id.0 == 0 {
        return Err(SaveError::SnapshotInvariant("command id must be nonzero"));
    }
    if !command.submitted_on.is_valid() || !command.execute_on.is_valid() {
        return Err(SaveError::SnapshotInvariant(
            "command contains invalid date",
        ));
    }
    if command.submitted_on > world.date {
        return Err(SaveError::SnapshotInvariant(
            "command submission date is after current world date",
        ));
    }
    if command.execute_on < command.submitted_on {
        return Err(SaveError::SnapshotInvariant(
            "command execution date precedes submission date",
        ));
    }
    if command.priority != CommandPriority::for_source(command.source) {
        return Err(SaveError::SnapshotInvariant(
            "command priority does not match command source",
        ));
    }
    Ok(())
}

fn validate_execution_record(
    record: &CommandExecutionRecord,
    world: &WorldState,
) -> Result<(), SaveError> {
    validate_command(&record.command, world)?;
    if !record.executed_on.is_valid() {
        return Err(SaveError::SnapshotInvariant(
            "executed command contains invalid execution date",
        ));
    }
    if record.executed_on != record.command.execute_on {
        return Err(SaveError::SnapshotInvariant(
            "executed command date differs from resolved execution date",
        ));
    }
    if record.executed_on >= world.date {
        return Err(SaveError::SnapshotInvariant(
            "executed command is not earlier than current world date",
        ));
    }
    Ok(())
}

fn validate_events(
    snapshot: &EngineSnapshot,
    executed_sources: &BTreeMap<CommandId, CommandSource>,
) -> Result<(), SaveError> {
    let mut expected_event_id = 1_u64;
    let mut command_event_ids = BTreeSet::new();

    for event in &snapshot.events {
        if event.id != EventId(expected_event_id) {
            return Err(SaveError::SnapshotInvariant(
                "event ids are not contiguous append order",
            ));
        }
        expected_event_id = expected_event_id
            .checked_add(1)
            .ok_or(SaveError::SnapshotInvariant("event id overflow"))?;

        if !event.occurred_on.is_valid() || event.occurred_on >= snapshot.world.date {
            return Err(SaveError::SnapshotInvariant(
                "event occurrence date is outside completed simulation time",
            ));
        }
        if event.tick_index >= snapshot.world.elapsed_days {
            return Err(SaveError::SnapshotInvariant(
                "event tick index is outside completed simulation time",
            ));
        }

        match event.payload {
            EventPayload::CommandExecuted { command_id } => {
                let source =
                    executed_sources
                        .get(&command_id)
                        .ok_or(SaveError::SnapshotInvariant(
                            "command-executed event references unknown executed command",
                        ))?;
                if !command_event_ids.insert(command_id) {
                    return Err(SaveError::SnapshotInvariant(
                        "executed command has duplicate execution events",
                    ));
                }
                validate_command_event_attribution(event, *source)?;
            }
        }
    }

    if command_event_ids.len() != snapshot.executed_commands.len() {
        return Err(SaveError::SnapshotInvariant(
            "executed command history and execution events disagree",
        ));
    }

    if snapshot.next_event_id != expected_event_id || snapshot.next_event_id == 0 {
        return Err(SaveError::SnapshotInvariant(
            "next event id does not follow append-only event history",
        ));
    }

    Ok(())
}

fn validate_command_event_attribution(
    event: &EventRecord,
    command_source: CommandSource,
) -> Result<(), SaveError> {
    let valid = match command_source {
        CommandSource::User => {
            event.category == EventCategory::UserIntervention && event.source == EventSource::User
        }
        CommandSource::CountryAi(country_id) => {
            event.category == EventCategory::System
                && event.source == EventSource::Country(country_id)
        }
    };

    if !valid {
        return Err(SaveError::SnapshotInvariant(
            "command execution event attribution is inconsistent",
        ));
    }
    Ok(())
}

fn validate_metadata_matches_snapshot(
    metadata: &SaveMetadata,
    snapshot: &EngineSnapshot,
) -> Result<(), SaveError> {
    let metadata_date = SimulationDate::new(
        metadata.simulation_year,
        metadata.simulation_month,
        metadata.simulation_day,
    );
    if metadata_date != snapshot.world.date {
        return Err(SaveError::InvalidMetadata(
            "metadata date does not match binary state",
        ));
    }
    if metadata.elapsed_days != snapshot.world.elapsed_days {
        return Err(SaveError::InvalidMetadata(
            "metadata elapsed day counter does not match binary state",
        ));
    }

    let metadata_seed = parse_hex_u64(&metadata.seed_hex, "invalid seed encoding")?;
    if metadata_seed != snapshot.world.seed {
        return Err(SaveError::InvalidMetadata(
            "metadata seed does not match binary state",
        ));
    }
    Ok(())
}

fn command_key(command: &QueuedCommand) -> (SimulationDate, CommandPriority, CommandId) {
    (command.execute_on, command.priority, command.id)
}

fn execution_key(record: &CommandExecutionRecord) -> (SimulationDate, CommandPriority, CommandId) {
    (
        record.executed_on,
        record.command.priority,
        record.command.id,
    )
}

fn parse_hex_u64(value: &str, message: &'static str) -> Result<u64, SaveError> {
    if value.len() != 16 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(SaveError::InvalidMetadata(message));
    }
    u64::from_str_radix(value, 16).map_err(|_| SaveError::InvalidMetadata(message))
}

#[cfg(test)]
mod tests {
    use simulation_model::WorldState;

    use super::{
        create_bundle, decode_bundle, AutosavePolicy, EngineSnapshot, SaveError, SaveKind,
        SAVE_FORMAT_VERSION,
    };

    fn empty_snapshot(seed: u64) -> EngineSnapshot {
        EngineSnapshot {
            world: WorldState::new(seed),
            pending_commands: Vec::new(),
            executed_commands: Vec::new(),
            events: Vec::new(),
            next_command_id: 1,
            next_event_id: 1,
        }
    }

    #[test]
    fn metadata_and_binary_round_trip_exactly() {
        let snapshot = empty_snapshot(0xfedc_ba98_7654_3210);
        let bundle =
            create_bundle(&snapshot, SaveKind::Manual).expect("valid snapshot should save");
        let metadata = bundle.metadata().expect("metadata should parse");

        assert_eq!(metadata.format_version, SAVE_FORMAT_VERSION);
        assert_eq!(metadata.save_kind, SaveKind::Manual);
        assert_eq!(metadata.seed_hex, "fedcba9876543210");

        let (kind, decoded) = decode_bundle(&bundle).expect("bundle should restore");
        assert_eq!(kind, SaveKind::Manual);
        assert_eq!(decoded, snapshot);
    }

    #[test]
    fn binary_corruption_is_rejected_by_checksum() {
        let snapshot = empty_snapshot(7);
        let mut bundle =
            create_bundle(&snapshot, SaveKind::Manual).expect("valid snapshot should save");
        let last = bundle
            .state_binary
            .last_mut()
            .expect("binary snapshot is nonempty");
        *last ^= 0x80;

        assert!(matches!(
            decode_bundle(&bundle),
            Err(SaveError::ChecksumMismatch { .. })
        ));
    }

    #[test]
    fn unsupported_metadata_version_is_rejected_before_state_decode() {
        let snapshot = empty_snapshot(7);
        let mut bundle =
            create_bundle(&snapshot, SaveKind::Manual).expect("valid snapshot should save");
        let mut metadata: serde_json::Value =
            serde_json::from_str(&bundle.metadata_json).expect("metadata should parse");
        metadata["format_version"] = serde_json::Value::from(999_u64);
        bundle.metadata_json =
            serde_json::to_string_pretty(&metadata).expect("metadata should encode");

        assert_eq!(
            decode_bundle(&bundle),
            Err(SaveError::UnsupportedMetadataVersion { found: 999 })
        );
    }

    #[test]
    fn autosave_policy_requires_positive_interval_and_tracks_successful_save_day() {
        assert_eq!(
            AutosavePolicy::new(0),
            Err(SaveError::InvalidAutosaveInterval)
        );
        let policy = AutosavePolicy::new(30).expect("positive interval should be valid");
        assert!(!policy.is_due(29, None));
        assert!(policy.is_due(30, None));
        assert!(!policy.is_due(59, Some(30)));
        assert!(policy.is_due(60, Some(30)));
    }

    #[test]
    fn inconsistent_next_id_is_rejected() {
        let mut snapshot = empty_snapshot(7);
        snapshot.next_command_id = 2;
        assert!(matches!(
            create_bundle(&snapshot, SaveKind::Manual),
            Err(SaveError::SnapshotInvariant(_))
        ));
    }
}
