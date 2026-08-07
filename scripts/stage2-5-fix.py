from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text()
    if old not in text:
        raise SystemExit(f'anchor not found in {path}: {old[:100]!r}')
    p.write_text(text.replace(old, new, 1))

# Entity errors can be persisted as typed rejection facts.
replace_once(
    'crates/simulation-model/src/entity.rs',
    '#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub enum EntityRegistryError {',
    '#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]\npub enum EntityRegistryError {'
)

# Event Ledger gains explicit rejected lifecycle facts.
replace_once(
    'crates/simulation-model/src/event.rs',
    'use crate::{CommandId, CountryId, EntityRef, SimulationDate};',
    'use crate::{CommandId, CountryId, EntityRef, EntityRegistryError, SimulationDate};'
)
replace_once(
    'crates/simulation-model/src/event.rs',
    '''    EntityCreated { entity: EntityRef },
    EntityRemoved { entity: EntityRef },
}''',
    '''    EntityCreated { entity: EntityRef },
    EntityRemoved { entity: EntityRef },
    EntityMutationRejected { error: EntityRegistryError },
}'''
)

# Command payload classification and source rejection.
replace_once(
    'crates/simulation-model/src/lib.rs',
    '''impl CommandPayload {
    #[must_use]
    pub fn references_country(&self, country_id: CountryId) -> bool {''',
    '''impl CommandPayload {
    #[must_use]
    pub const fn is_entity_lifecycle(&self) -> bool {
        !matches!(self, Self::NoOp { .. })
    }

    #[must_use]
    pub fn references_country(&self, country_id: CountryId) -> bool {'''
)
replace_once(
    'crates/simulation-model/src/lib.rs',
    '''    CommandIdExhausted,
}''',
    '''    CommandIdExhausted,
    InvalidSourceForPayload,
}'''
)
replace_once(
    'crates/simulation-model/src/lib.rs',
    '''            Self::CommandIdExhausted => formatter.write_str("command identifier space exhausted"),
        }
    }
}''',
    '''            Self::CommandIdExhausted => formatter.write_str("command identifier space exhausted"),
            Self::InvalidSourceForPayload => {
                formatter.write_str("entity lifecycle payloads require system command source")
            }
        }
    }
}'''
)

# Core submission and execution semantics.
replace_once(
    'crates/simulation-core/src/lib.rs',
    '''    pub fn submit_command(&mut self, request: CommandRequest) -> Result<CommandId, CommandError> {
        let execute_on = self.resolve_execution_date(request.timing)?;''',
    '''    pub fn submit_command(&mut self, request: CommandRequest) -> Result<CommandId, CommandError> {
        if request.payload.is_entity_lifecycle() && request.source != CommandSource::System {
            return Err(CommandError::InvalidSourceForPayload);
        }
        let execute_on = self.resolve_execution_date(request.timing)?;'''
)
replace_once(
    'crates/simulation-core/src/lib.rs',
    '''        let lifecycle_event = self.apply_entity_payload(&command.payload).ok().flatten();

        self.executed_commands.push(CommandExecutionRecord {''',
    '''        let lifecycle_result = self.apply_entity_payload(&command.payload);

        self.executed_commands.push(CommandExecutionRecord {'''
)
replace_once(
    'crates/simulation-core/src/lib.rs',
    '''        if let Some(payload) = lifecycle_event {
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
''',
    '''        match lifecycle_result {
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
'''
)

# Save invariant validation accepts explicit rejection facts.
replace_once(
    'crates/simulation-save/src/lib.rs',
    '''            EventPayload::EntityCreated { entity } | EventPayload::EntityRemoved { entity } => {
                if event.category != EventCategory::EntityLifecycle
                    || event.source != EventSource::System
                    || !entity.id.is_valid()
                {
                    return Err(SaveError::SnapshotInvariant(
                        "entity lifecycle event attribution is invalid",
                    ));
                }
            }
''',
    '''            EventPayload::EntityCreated { entity } | EventPayload::EntityRemoved { entity } => {
                if event.category != EventCategory::EntityLifecycle
                    || event.source != EventSource::System
                    || !entity.id.is_valid()
                {
                    return Err(SaveError::SnapshotInvariant(
                        "entity lifecycle event attribution is invalid",
                    ));
                }
            }
            EventPayload::EntityMutationRejected { .. } => {
                if event.category != EventCategory::EntityLifecycle
                    || event.source != EventSource::System
                {
                    return Err(SaveError::SnapshotInvariant(
                        "entity mutation rejection attribution is invalid",
                    ));
                }
            }
'''
)

# Binary import includes error type.
replace_once(
    'crates/simulation-save/src/binary.rs',
    '''    EntityKind, EntityRef, EntityRegistry, EntityWorldState, EventCategory, EventId, EventPayload,
''',
    '''    EntityKind, EntityRef, EntityRegistry, EntityRegistryError, EntityWorldState, EventCategory,
    EventId, EventPayload,
'''
)

# Make queued-command encoding fallible end-to-end.
replace_once(
    'crates/simulation-save/src/binary.rs',
    '''    for command in &snapshot.pending_commands {
        write_queued_command(&mut bytes, command);
    }''',
    '''    for command in &snapshot.pending_commands {
        write_queued_command(&mut bytes, command)?;
    }'''
)
replace_once(
    'crates/simulation-save/src/binary.rs',
    '''    for record in &snapshot.executed_commands {
        write_execution_record(&mut bytes, record);
    }''',
    '''    for record in &snapshot.executed_commands {
        write_execution_record(&mut bytes, record)?;
    }'''
)
replace_once(
    'crates/simulation-save/src/binary.rs',
    '''fn write_queued_command(bytes: &mut Vec<u8>, command: &QueuedCommand) {
    write_u64(bytes, command.id.0);
    write_command_source(bytes, command.source);
    write_date(bytes, command.submitted_on);
    write_date(bytes, command.execute_on);
    write_command_priority(bytes, command.priority);
    write_command_payload(bytes, &command.payload);
}''',
    '''fn write_queued_command(bytes: &mut Vec<u8>, command: &QueuedCommand) -> Result<(), SaveError> {
    write_u64(bytes, command.id.0);
    write_command_source(bytes, command.source);
    write_date(bytes, command.submitted_on);
    write_date(bytes, command.execute_on);
    write_command_priority(bytes, command.priority);
    write_command_payload(bytes, &command.payload)?;
    Ok(())
}'''
)
replace_once(
    'crates/simulation-save/src/binary.rs',
    '''fn write_execution_record(bytes: &mut Vec<u8>, record: &CommandExecutionRecord) {
    write_queued_command(bytes, &record.command);
    write_date(bytes, record.executed_on);
}''',
    '''fn write_execution_record(
    bytes: &mut Vec<u8>,
    record: &CommandExecutionRecord,
) -> Result<(), SaveError> {
    write_queued_command(bytes, &record.command)?;
    write_date(bytes, record.executed_on);
    Ok(())
}'''
)
replace_once(
    'crates/simulation-save/src/binary.rs',
    'fn write_command_payload(bytes: &mut Vec<u8>, payload: &CommandPayload) {',
    'fn write_command_payload(bytes: &mut Vec<u8>, payload: &CommandPayload) -> Result<(), SaveError> {'
)
replace_once(
    'crates/simulation-save/src/binary.rs',
    '''        CommandPayload::CreateRegionEntity { draft } => {
            write_u8(bytes, 5);
            let _ = write_region_draft(bytes, draft);
        }''',
    '''        CommandPayload::CreateRegionEntity { draft } => {
            write_u8(bytes, 5);
            write_region_draft(bytes, draft)?;
        }'''
)
replace_once(
    'crates/simulation-save/src/binary.rs',
    '''        CommandPayload::RemoveRegionEntity { region_id } => {
            write_u8(bytes, 6);
            write_u64(bytes, region_id.0);
        }
    }
}''',
    '''        CommandPayload::RemoveRegionEntity { region_id } => {
            write_u8(bytes, 6);
            write_u64(bytes, region_id.0);
        }
    }
    Ok(())
}'''
)

# Event rejection codec.
replace_once(
    'crates/simulation-save/src/binary.rs',
    '''        EventPayload::EntityRemoved { entity } => {
            write_u8(bytes, 2);
            write_entity_ref(bytes, entity);
        }
    }
}''',
    '''        EventPayload::EntityRemoved { entity } => {
            write_u8(bytes, 2);
            write_entity_ref(bytes, entity);
        }
        EventPayload::EntityMutationRejected { error } => {
            write_u8(bytes, 3);
            write_entity_registry_error(bytes, error);
        }
    }
}'''
)
anchor = '''fn write_entity_ref(bytes: &mut Vec<u8>, entity: EntityRef) {'''
helpers = '''fn write_entity_registry_error(bytes: &mut Vec<u8>, error: EntityRegistryError) {
    write_u8(
        bytes,
        match error {
            EntityRegistryError::InvalidId => 0,
            EntityRegistryError::DuplicateId => 1,
            EntityRegistryError::NonMonotonicNextId => 2,
            EntityRegistryError::IdExhausted => 3,
            EntityRegistryError::UnknownEntity => 4,
            EntityRegistryError::ReferenceInUse => 5,
            EntityRegistryError::InvalidReference => 6,
        },
    );
}

fn read_entity_registry_error(cursor: &mut Cursor<'_>) -> Result<EntityRegistryError, SaveError> {
    match cursor.read_u8()? {
        0 => Ok(EntityRegistryError::InvalidId),
        1 => Ok(EntityRegistryError::DuplicateId),
        2 => Ok(EntityRegistryError::NonMonotonicNextId),
        3 => Ok(EntityRegistryError::IdExhausted),
        4 => Ok(EntityRegistryError::UnknownEntity),
        5 => Ok(EntityRegistryError::ReferenceInUse),
        6 => Ok(EntityRegistryError::InvalidReference),
        tag => Err(SaveError::InvalidTag {
            field: "entity registry error",
            tag,
        }),
    }
}

'''+anchor
replace_once('crates/simulation-save/src/binary.rs', anchor, helpers)
replace_once(
    'crates/simulation-save/src/binary.rs',
    '''        2 => Ok(EventPayload::EntityRemoved {
            entity: read_entity_ref(cursor)?,
        }),
        tag => Err(SaveError::InvalidTag {''',
    '''        2 => Ok(EventPayload::EntityRemoved {
            entity: read_entity_ref(cursor)?,
        }),
        3 => Ok(EventPayload::EntityMutationRejected {
            error: read_entity_registry_error(cursor)?,
        }),
        tag => Err(SaveError::InvalidTag {'''
)

print('Stage 2.5 rejection semantics and fallible save encoding patched')
