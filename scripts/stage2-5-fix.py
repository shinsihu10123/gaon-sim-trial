from pathlib import Path

path = Path('crates/simulation-save/src/lib.rs')
text = path.read_text()
old = '''    let valid = match command_source {
        CommandSource::User => {
            event.category == EventCategory::UserIntervention && event.source == EventSource::User
        }
'''
new = '''    let valid = match command_source {
        CommandSource::System => {
            event.category == EventCategory::System && event.source == EventSource::System
        }
        CommandSource::User => {
            event.category == EventCategory::UserIntervention && event.source == EventSource::User
        }
'''
if old not in text:
    raise SystemExit('command attribution anchor not found')
path.write_text(text.replace(old, new, 1))
