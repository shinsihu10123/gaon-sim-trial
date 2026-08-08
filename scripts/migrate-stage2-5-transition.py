from pathlib import Path


def replace_once(path: str, old: str, new: str, already: str) -> None:
    file = Path(path)
    text = file.read_text()
    if already in text:
        return
    if old not in text:
        raise SystemExit(f"transition migration anchor missing in {path}:\n{old[:180]}")
    file.write_text(text.replace(old, new, 1))


replace_once(
    "crates/simulation-model/src/lib.rs",
    """    RemovePoliticalEntity {
        political_entity_id: PoliticalEntityId,
    },
    CreateCountryEntity,""",
    """    RemovePoliticalEntity {
        political_entity_id: PoliticalEntityId,
    },
    PromotePoliticalEntityToCountry {
        political_entity_id: PoliticalEntityId,
    },
    CreateCountryEntity,""",
    "PromotePoliticalEntityToCountry",
)

replace_once(
    "crates/simulation-model/src/lib.rs",
    """            | Self::CreatePoliticalEntity
            | Self::RemovePoliticalEntity { .. }
            | Self::CreateCountryEntity""",
    """            | Self::CreatePoliticalEntity
            | Self::RemovePoliticalEntity { .. }
            | Self::PromotePoliticalEntityToCountry { .. }
            | Self::CreateCountryEntity""",
    "Self::PromotePoliticalEntityToCountry { .. }",
)

# references_region has a second exhaustive no-reference arm; patch it separately
path = Path("crates/simulation-model/src/lib.rs")
text = path.read_text()
needle = "| Self::RemovePoliticalEntity { .. }\n            | Self::CreateCountryEntity"
if text.count(needle) == 1:
    text = text.replace(
        needle,
        "| Self::RemovePoliticalEntity { .. }\n            | Self::PromotePoliticalEntityToCountry { .. }\n            | Self::CreateCountryEntity",
        1,
    )
    path.write_text(text)

replace_once(
    "crates/simulation-model/src/event.rs",
    """    EntityCreated { entity: EntityRef },
    EntityRemoved { entity: EntityRef },
    EntityMutationRejected { error: EntityRegistryError },""",
    """    EntityCreated { entity: EntityRef },
    EntityRemoved { entity: EntityRef },
    EntityTransitioned { from: EntityRef, to: EntityRef },
    EntityMutationRejected { error: EntityRegistryError },""",
    "EntityTransitioned",
)

replace_once(
    "crates/simulation-core/src/entity_lifecycle.rs",
    """            CommandPayload::RemovePoliticalEntity {
                political_entity_id,
            } => {
                self.state
                    .entities
                    .political_entities
                    .remove(*political_entity_id)?;
                Ok(Some(EventPayload::EntityRemoved {
                    entity: EntityRef::political_entity(*political_entity_id),
                }))
            }
            CommandPayload::CreateCountryEntity => {""",
    """            CommandPayload::RemovePoliticalEntity {
                political_entity_id,
            } => {
                self.state
                    .entities
                    .political_entities
                    .remove(*political_entity_id)?;
                Ok(Some(EventPayload::EntityRemoved {
                    entity: EntityRef::political_entity(*political_entity_id),
                }))
            }
            CommandPayload::PromotePoliticalEntityToCountry {
                political_entity_id,
            } => {
                if !self
                    .state
                    .entities
                    .political_entities
                    .contains(*political_entity_id)
                {
                    return Err(EntityRegistryError::UnknownEntity);
                }
                let country_id = self.state.entities.countries.create()?;
                self.state
                    .entities
                    .political_entities
                    .remove(*political_entity_id)?;
                Ok(Some(EventPayload::EntityTransitioned {
                    from: EntityRef::political_entity(*political_entity_id),
                    to: EntityRef::country(country_id),
                }))
            }
            CommandPayload::CreateCountryEntity => {""",
    "CommandPayload::PromotePoliticalEntityToCountry",
)

replace_once(
    "crates/simulation-save/src/binary.rs",
    """        CommandPayload::RemovePoliticalEntity {
            political_entity_id,
        } => {
            write_u8(bytes, 14);
            write_u64(bytes, political_entity_id.0);
        }
    }""",
    """        CommandPayload::RemovePoliticalEntity {
            political_entity_id,
        } => {
            write_u8(bytes, 14);
            write_u64(bytes, political_entity_id.0);
        }
        CommandPayload::PromotePoliticalEntityToCountry {
            political_entity_id,
        } => {
            write_u8(bytes, 15);
            write_u64(bytes, political_entity_id.0);
        }
    }""",
    "write_u8(bytes, 15)",
)

replace_once(
    "crates/simulation-save/src/binary.rs",
    """        14 => Ok(CommandPayload::RemovePoliticalEntity {
            political_entity_id: PoliticalEntityId(cursor.read_u64()?),
        }),
        tag => Err(SaveError::InvalidTag {""",
    """        14 => Ok(CommandPayload::RemovePoliticalEntity {
            political_entity_id: PoliticalEntityId(cursor.read_u64()?),
        }),
        15 => Ok(CommandPayload::PromotePoliticalEntityToCountry {
            political_entity_id: PoliticalEntityId(cursor.read_u64()?),
        }),
        tag => Err(SaveError::InvalidTag {""",
    "15 => Ok(CommandPayload::PromotePoliticalEntityToCountry",
)

replace_once(
    "crates/simulation-save/src/binary.rs",
    """        EventPayload::EntityRemoved { entity } => {
            write_u8(bytes, 2);
            write_entity_ref(bytes, entity);
        }
        EventPayload::EntityMutationRejected { error } => {
            write_u8(bytes, 3);
            write_entity_registry_error(bytes, error);
        }""",
    """        EventPayload::EntityRemoved { entity } => {
            write_u8(bytes, 2);
            write_entity_ref(bytes, entity);
        }
        EventPayload::EntityMutationRejected { error } => {
            write_u8(bytes, 3);
            write_entity_registry_error(bytes, error);
        }
        EventPayload::EntityTransitioned { from, to } => {
            write_u8(bytes, 4);
            write_entity_ref(bytes, from);
            write_entity_ref(bytes, to);
        }""",
    "EventPayload::EntityTransitioned { from, to }",
)

replace_once(
    "crates/simulation-save/src/binary.rs",
    """        3 => Ok(EventPayload::EntityMutationRejected {
            error: read_entity_registry_error(cursor)?,
        }),
        tag => Err(SaveError::InvalidTag {""",
    """        3 => Ok(EventPayload::EntityMutationRejected {
            error: read_entity_registry_error(cursor)?,
        }),
        4 => Ok(EventPayload::EntityTransitioned {
            from: read_entity_ref(cursor)?,
            to: read_entity_ref(cursor)?,
        }),
        tag => Err(SaveError::InvalidTag {""",
    "4 => Ok(EventPayload::EntityTransitioned",
)
