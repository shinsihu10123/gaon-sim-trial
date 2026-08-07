from pathlib import Path

path = Path('crates/simulation-model/src/world.rs')
text = path.read_text()
old = '''        assert!(spatial
            .region(RegionId(2))
            .expect("region 2")
            .neighbors
            .is_empty());
'''
new = '''        assert_eq!(
            spatial.region(RegionId(2)).expect("region 2").neighbors,
            vec![RegionId(1)]
        );
'''
if old not in text:
    raise SystemExit('first adjacency expectation anchor not found')
text = text.replace(old, new, 1)
old = '''        assert_eq!(
            spatial.region(RegionId(2)).expect("region 2").neighbors,
            vec![RegionId(4)]
        );
'''
new = '''        assert_eq!(
            spatial.region(RegionId(2)).expect("region 2").neighbors,
            vec![RegionId(1), RegionId(4)]
        );
'''
if old not in text:
    raise SystemExit('second adjacency expectation anchor not found')
path.write_text(text.replace(old, new, 1))
