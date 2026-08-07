use simulation_model::{
    CommandExecutionRecord, CommandId, CommandPayload, CommandPriority, CommandSource, CountryId,
    EventCategory, EventId, EventPayload, EventRecord, EventSource, QueuedCommand, SimulationDate,
    WorldState,
};

use crate::{EngineSnapshot, SaveError, SAVE_FORMAT_VERSION};

const STATE_MAGIC: [u8; 8] = *b"GAONST01";
const MAX_RECORD_COUNT: u64 = 10_000_000;

pub(crate) fn encode_snapshot(snapshot: &EngineSnapshot) -> Result<Vec<u8>, SaveError> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&STATE_MAGIC);
    write_u32(&mut bytes, SAVE_FORMAT_VERSION);
    write_world(&mut bytes, &snapshot.world);
    write_u64(&mut bytes, snapshot.next_command_id);
    write_u64(&mut bytes, snapshot.next_event_id);

    write_count(
        &mut bytes,
        "pending_commands",
        snapshot.pending_commands.len(),
    )?;
    for command in &snapshot.pending_commands {
        write_queued_command(&mut bytes, command);
    }

    write_count(
        &mut bytes,
        "executed_commands",
        snapshot.executed_commands.len(),
    )?;
    for record in &snapshot.executed_commands {
        write_execution_record(&mut bytes, record);
    }

    write_count(&mut bytes, "events", snapshot.events.len())?;
    for event in &snapshot.events {
        write_event_record(&mut bytes, event);
    }

    Ok(bytes)
}

pub(crate) fn decode_snapshot(bytes: &[u8]) -> Result<EngineSnapshot, SaveError> {
    let mut cursor = Cursor::new(bytes);
    let magic = cursor.read_array::<8>()?;
    if magic != STATE_MAGIC {
        return Err(SaveError::InvalidBinary("invalid state magic"));
    }

    let version = cursor.read_u32()?;
    if version != SAVE_FORMAT_VERSION {
        return Err(SaveError::UnsupportedBinaryVersion { found: version });
    }

    let world = read_world(&mut cursor)?;
    let next_command_id = cursor.read_u64()?;
    let next_event_id = cursor.read_u64()?;

    let pending_count = cursor.read_count("pending_commands")?;
    let mut pending_commands = Vec::with_capacity(pending_count);
    for _ in 0..pending_count {
        pending_commands.push(read_queued_command(&mut cursor)?);
    }

    let executed_count = cursor.read_count("executed_commands")?;
    let mut executed_commands = Vec::with_capacity(executed_count);
    for _ in 0..executed_count {
        executed_commands.push(read_execution_record(&mut cursor)?);
    }

    let event_count = cursor.read_count("events")?;
    let mut events = Vec::with_capacity(event_count);
    for _ in 0..event_count {
        events.push(read_event_record(&mut cursor)?);
    }

    if !cursor.is_finished() {
        return Err(SaveError::TrailingBinaryData);
    }

    Ok(EngineSnapshot {
        world,
        pending_commands,
        executed_commands,
        events,
        next_command_id,
        next_event_id,
    })
}

fn write_world(bytes: &mut Vec<u8>, world: &WorldState) {
    write_date(bytes, world.date);
    write_u64(bytes, world.elapsed_days);
    write_u64(bytes, world.seed);
}

fn read_world(cursor: &mut Cursor<'_>) -> Result<WorldState, SaveError> {
    let date = read_date(cursor)?;
    let elapsed_days = cursor.read_u64()?;
    let seed = cursor.read_u64()?;
    Ok(WorldState {
        date,
        elapsed_days,
        seed,
    })
}

fn write_queued_command(bytes: &mut Vec<u8>, command: &QueuedCommand) {
    write_u64(bytes, command.id.0);
    write_command_source(bytes, command.source);
    write_date(bytes, command.submitted_on);
    write_date(bytes, command.execute_on);
    write_command_priority(bytes, command.priority);
    write_command_payload(bytes, &command.payload);
}

fn read_queued_command(cursor: &mut Cursor<'_>) -> Result<QueuedCommand, SaveError> {
    Ok(QueuedCommand {
        id: CommandId(cursor.read_u64()?),
        source: read_command_source(cursor)?,
        submitted_on: read_date(cursor)?,
        execute_on: read_date(cursor)?,
        priority: read_command_priority(cursor)?,
        payload: read_command_payload(cursor)?,
    })
}

fn write_execution_record(bytes: &mut Vec<u8>, record: &CommandExecutionRecord) {
    write_queued_command(bytes, &record.command);
    write_date(bytes, record.executed_on);
}

fn read_execution_record(cursor: &mut Cursor<'_>) -> Result<CommandExecutionRecord, SaveError> {
    Ok(CommandExecutionRecord {
        command: read_queued_command(cursor)?,
        executed_on: read_date(cursor)?,
    })
}

fn write_event_record(bytes: &mut Vec<u8>, event: &EventRecord) {
    write_u64(bytes, event.id.0);
    write_date(bytes, event.occurred_on);
    write_u64(bytes, event.tick_index);
    write_event_category(bytes, event.category);
    write_event_source(bytes, event.source);
    write_event_payload(bytes, event.payload);
}

fn read_event_record(cursor: &mut Cursor<'_>) -> Result<EventRecord, SaveError> {
    Ok(EventRecord {
        id: EventId(cursor.read_u64()?),
        occurred_on: read_date(cursor)?,
        tick_index: cursor.read_u64()?,
        category: read_event_category(cursor)?,
        source: read_event_source(cursor)?,
        payload: read_event_payload(cursor)?,
    })
}

fn write_command_source(bytes: &mut Vec<u8>, source: CommandSource) {
    match source {
        CommandSource::User => write_u8(bytes, 0),
        CommandSource::CountryAi(country_id) => {
            write_u8(bytes, 1);
            write_u16(bytes, country_id.0);
        }
    }
}

fn read_command_source(cursor: &mut Cursor<'_>) -> Result<CommandSource, SaveError> {
    match cursor.read_u8()? {
        0 => Ok(CommandSource::User),
        1 => Ok(CommandSource::CountryAi(CountryId(cursor.read_u16()?))),
        tag => Err(SaveError::InvalidTag {
            field: "command source",
            tag,
        }),
    }
}

fn write_command_priority(bytes: &mut Vec<u8>, priority: CommandPriority) {
    let tag = match priority {
        CommandPriority::UserIntervention => 10,
        CommandPriority::CountryAi => 20,
    };
    write_u8(bytes, tag);
}

fn read_command_priority(cursor: &mut Cursor<'_>) -> Result<CommandPriority, SaveError> {
    match cursor.read_u8()? {
        10 => Ok(CommandPriority::UserIntervention),
        20 => Ok(CommandPriority::CountryAi),
        tag => Err(SaveError::InvalidTag {
            field: "command priority",
            tag,
        }),
    }
}

fn write_command_payload(bytes: &mut Vec<u8>, payload: &CommandPayload) {
    match payload {
        CommandPayload::NoOp { token } => {
            write_u8(bytes, 0);
            write_u64(bytes, *token);
        }
    }
}

fn read_command_payload(cursor: &mut Cursor<'_>) -> Result<CommandPayload, SaveError> {
    match cursor.read_u8()? {
        0 => Ok(CommandPayload::NoOp {
            token: cursor.read_u64()?,
        }),
        tag => Err(SaveError::InvalidTag {
            field: "command payload",
            tag,
        }),
    }
}

fn write_event_category(bytes: &mut Vec<u8>, category: EventCategory) {
    let tag = match category {
        EventCategory::System => 0,
        EventCategory::Economic => 1,
        EventCategory::Policy => 2,
        EventCategory::Diplomatic => 3,
        EventCategory::Military => 4,
        EventCategory::War => 5,
        EventCategory::Occupation => 6,
        EventCategory::Treaty => 7,
        EventCategory::UserIntervention => 8,
    };
    write_u8(bytes, tag);
}

fn read_event_category(cursor: &mut Cursor<'_>) -> Result<EventCategory, SaveError> {
    match cursor.read_u8()? {
        0 => Ok(EventCategory::System),
        1 => Ok(EventCategory::Economic),
        2 => Ok(EventCategory::Policy),
        3 => Ok(EventCategory::Diplomatic),
        4 => Ok(EventCategory::Military),
        5 => Ok(EventCategory::War),
        6 => Ok(EventCategory::Occupation),
        7 => Ok(EventCategory::Treaty),
        8 => Ok(EventCategory::UserIntervention),
        tag => Err(SaveError::InvalidTag {
            field: "event category",
            tag,
        }),
    }
}

fn write_event_source(bytes: &mut Vec<u8>, source: EventSource) {
    match source {
        EventSource::System => write_u8(bytes, 0),
        EventSource::User => write_u8(bytes, 1),
        EventSource::Country(country_id) => {
            write_u8(bytes, 2);
            write_u16(bytes, country_id.0);
        }
    }
}

fn read_event_source(cursor: &mut Cursor<'_>) -> Result<EventSource, SaveError> {
    match cursor.read_u8()? {
        0 => Ok(EventSource::System),
        1 => Ok(EventSource::User),
        2 => Ok(EventSource::Country(CountryId(cursor.read_u16()?))),
        tag => Err(SaveError::InvalidTag {
            field: "event source",
            tag,
        }),
    }
}

fn write_event_payload(bytes: &mut Vec<u8>, payload: EventPayload) {
    match payload {
        EventPayload::CommandExecuted { command_id } => {
            write_u8(bytes, 0);
            write_u64(bytes, command_id.0);
        }
    }
}

fn read_event_payload(cursor: &mut Cursor<'_>) -> Result<EventPayload, SaveError> {
    match cursor.read_u8()? {
        0 => Ok(EventPayload::CommandExecuted {
            command_id: CommandId(cursor.read_u64()?),
        }),
        tag => Err(SaveError::InvalidTag {
            field: "event payload",
            tag,
        }),
    }
}

fn write_date(bytes: &mut Vec<u8>, date: SimulationDate) {
    write_u32(bytes, date.year);
    write_u8(bytes, date.month);
    write_u8(bytes, date.day);
}

fn read_date(cursor: &mut Cursor<'_>) -> Result<SimulationDate, SaveError> {
    let date = SimulationDate::new(cursor.read_u32()?, cursor.read_u8()?, cursor.read_u8()?);
    if !date.is_valid() {
        return Err(SaveError::InvalidBinary("invalid simulation date"));
    }
    Ok(date)
}

fn write_count(bytes: &mut Vec<u8>, section: &'static str, count: usize) -> Result<(), SaveError> {
    let count = u64::try_from(count).map_err(|_| SaveError::RecordCountOverflow { section })?;
    write_u64(bytes, count);
    Ok(())
}

fn write_u8(bytes: &mut Vec<u8>, value: u8) {
    bytes.push(value);
}

fn write_u16(bytes: &mut Vec<u8>, value: u16) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn write_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn write_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

struct Cursor<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> Cursor<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    const fn is_finished(&self) -> bool {
        self.position == self.bytes.len()
    }

    fn read_count(&mut self, section: &'static str) -> Result<usize, SaveError> {
        let count = self.read_u64()?;
        if count > MAX_RECORD_COUNT {
            return Err(SaveError::RecordLimitExceeded { section, count });
        }
        usize::try_from(count).map_err(|_| SaveError::RecordLimitExceeded { section, count })
    }

    fn read_u8(&mut self) -> Result<u8, SaveError> {
        Ok(self.read_array::<1>()?[0])
    }

    fn read_u16(&mut self) -> Result<u16, SaveError> {
        Ok(u16::from_le_bytes(self.read_array::<2>()?))
    }

    fn read_u32(&mut self) -> Result<u32, SaveError> {
        Ok(u32::from_le_bytes(self.read_array::<4>()?))
    }

    fn read_u64(&mut self) -> Result<u64, SaveError> {
        Ok(u64::from_le_bytes(self.read_array::<8>()?))
    }

    fn read_array<const N: usize>(&mut self) -> Result<[u8; N], SaveError> {
        let end = self
            .position
            .checked_add(N)
            .ok_or(SaveError::TruncatedBinary)?;
        let slice = self
            .bytes
            .get(self.position..end)
            .ok_or(SaveError::TruncatedBinary)?;
        let mut output = [0_u8; N];
        output.copy_from_slice(slice);
        self.position = end;
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use simulation_model::WorldState;

    use super::{decode_snapshot, encode_snapshot};
    use crate::EngineSnapshot;

    #[test]
    fn empty_snapshot_binary_round_trip_is_exact() {
        let snapshot = EngineSnapshot {
            world: WorldState::new(77),
            pending_commands: Vec::new(),
            executed_commands: Vec::new(),
            events: Vec::new(),
            next_command_id: 1,
            next_event_id: 1,
        };

        let encoded = encode_snapshot(&snapshot).expect("snapshot should encode");
        let decoded = decode_snapshot(&encoded).expect("snapshot should decode");
        assert_eq!(decoded, snapshot);
    }
}
