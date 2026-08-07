from pathlib import Path

path = Path('crates/simulation-save/src/binary.rs')
text = path.read_text()
old = '''    use simulation_model::{
        BiomeClass, MapPoint, RegionId, RegionPoliticalState, RegionState, RegionSurface,
        ReliefClass, TerrainSample, TerrainState, WorldBounds, WorldSpatialState, WorldState,
        TERRAIN_SAMPLE_COUNT, TRIAL_REGION_COUNT,
    };
'''
new = '''    use simulation_model::{
        BiomeClass, EntityRegistry, MapPoint, RegionId, RegionPoliticalState, RegionState,
        RegionSurface, ReliefClass, TerrainSample, TerrainState, WorldBounds, WorldSpatialState,
        WorldState, TERRAIN_SAMPLE_COUNT, TRIAL_REGION_COUNT,
    };
'''
if old not in text:
    raise SystemExit('binary test import anchor not found')
text = text.replace(old, new, 1)
text = text.replace('RegionId(u16::try_from(index).expect("id fits u16"))', 'RegionId(u64::try_from(index).expect("id fits u64"))')
text = text.replace('RegionId(u16::try_from(index - 1).expect("id fits u16"))', 'RegionId(u64::try_from(index - 1).expect("id fits u64"))')
text = text.replace('RegionId(u16::try_from(index + 1).expect("id fits u16"))', 'RegionId(u64::try_from(index + 1).expect("id fits u64"))')
path.write_text(text)
