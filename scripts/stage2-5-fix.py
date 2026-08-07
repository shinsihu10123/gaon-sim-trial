from pathlib import Path

path = Path('crates/simulation-save/src/lib.rs')
text = path.read_text()
old = '''use simulation_model::{
    CommandExecutionRecord, CommandId, CommandPriority, CommandSource, EventCategory, EventId,
    EventPayload, EventRecord, EventSource, QueuedCommand, SimulationDate, WorldState,
};
'''
new = '''use simulation_model::{
    CommandExecutionRecord, CommandId, CommandPriority, CommandSource, EntityRegistry,
    EventCategory, EventId, EventPayload, EventRecord, EventSource, QueuedCommand, SimulationDate,
    WorldState,
};
'''
if old not in text:
    raise SystemExit('top-level import anchor not found')
text = text.replace(old, new, 1)
text = text.replace('    use simulation_model::EntityRegistry as _;\n', '', 1)
text = text.replace('    for region in world.spatial.regions.iter() {\n', '    for region in &world.spatial.regions {\n', 1)
path.write_text(text)
