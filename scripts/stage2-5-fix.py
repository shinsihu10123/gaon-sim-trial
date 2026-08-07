from pathlib import Path

path = Path('crates/simulation-protocol/src/lib.rs')
text = path.read_text()
text = text.replace('RegionId(u16::try_from(index).expect("id fits u16"))', 'RegionId(u64::try_from(index).expect("id fits u64"))')
text = text.replace('RegionId(u16::try_from(index - 1).expect("id fits u16"))', 'RegionId(u64::try_from(index - 1).expect("id fits u64"))')
text = text.replace('RegionId(u16::try_from(index + 1).expect("id fits u16"))', 'RegionId(u64::try_from(index + 1).expect("id fits u64"))')
text = text.replace('assert_eq!(snapshot.world.regions[59].id, 60);', 'assert_eq!(snapshot.world.regions[59].id, "60");')
path.write_text(text)
