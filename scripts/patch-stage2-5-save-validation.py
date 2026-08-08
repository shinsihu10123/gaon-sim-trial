from pathlib import Path

path = Path("crates/simulation-save/src/lib.rs")
text = path.read_text()

if "entity transition event attribution is invalid" in text:
    raise SystemExit(0)

old = """            EventPayload::EntityCreated { entity } | EventPayload::EntityRemoved { entity } => {
                if event.category != EventCategory::EntityLifecycle
                    || event.source != EventSource::System
                    || !entity.id.is_valid()
                {
                    return Err(SaveError::SnapshotInvariant(
                        \"entity lifecycle event attribution is invalid\",
                    ));
                }
            }
            EventPayload::EntityMutationRejected { .. } => {"""

new = """            EventPayload::EntityCreated { entity } | EventPayload::EntityRemoved { entity } => {
                if event.category != EventCategory::EntityLifecycle
                    || event.source != EventSource::System
                    || !entity.id.is_valid()
                {
                    return Err(SaveError::SnapshotInvariant(
                        \"entity lifecycle event attribution is invalid\",
                    ));
                }
            }
            EventPayload::EntityTransitioned { from, to } => {
                if event.category != EventCategory::EntityLifecycle
                    || event.source != EventSource::System
                    || !from.id.is_valid()
                    || !to.id.is_valid()
                {
                    return Err(SaveError::SnapshotInvariant(
                        \"entity transition event attribution is invalid\",
                    ));
                }
            }
            EventPayload::EntityMutationRejected { .. } => {"""

if old not in text:
    raise SystemExit("save validation transition anchor missing")

path.write_text(text.replace(old, new, 1))
