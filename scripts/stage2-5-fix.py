from pathlib import Path

path = Path('crates/simulation-save/tests/spatial_canonical_state.rs')
text = path.read_text()
text = text.replace('u16::try_from(index).expect("trial id fits u16")', 'u64::try_from(index).expect("trial id fits u64")')
text = text.replace('u16::try_from(index - 1).expect("id fits u16")', 'u64::try_from(index - 1).expect("id fits u64")')
text = text.replace('u16::try_from(index + 1).expect("id fits u16")', 'u64::try_from(index + 1).expect("id fits u64")')
path.write_text(text)
