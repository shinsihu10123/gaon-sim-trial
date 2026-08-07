from pathlib import Path

path = Path('crates/simulation-model/tests/world_spatial_contract.rs')
text = path.read_text()
old = '''use simulation_model::{
    MapPoint, RegionId, RegionPoliticalState, RegionState, RegionSurface, WorldBounds,
    WorldSpatialError, WorldSpatialState, TRIAL_REGION_COUNT,
};
'''
new = '''use simulation_model::{
    EntityRegistry, MapPoint, RegionId, RegionPoliticalState, RegionState, RegionSurface,
    WorldBounds, WorldSpatialError, WorldSpatialState, TRIAL_REGION_COUNT,
};
'''
if old not in text:
    raise SystemExit('world spatial import anchor not found')
text = text.replace(old, new, 1)
text = text.replace('u16::try_from(index).expect("fixture id fits u16")', 'u64::try_from(index).expect("fixture id fits u64")')
text = text.replace('u16::try_from(index - 1).expect("id fits u16")', 'u64::try_from(index - 1).expect("id fits u64")')
text = text.replace('u16::try_from(index + 1).expect("id fits u16")', 'u64::try_from(index + 1).expect("id fits u64")')
path.write_text(text)
