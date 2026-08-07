# Stage 2.4 Validation Record

Status: **IN PROGRESS**

## Implemented in current work session

- `ResourceDeposit` finite-stock model
- separate initial/remaining quantity
- quality permille
- accessibility permille
- bounded extraction and depletion ratio
- renewable food-capacity representation
- `ResourceCellState` aligned to canonical terrain samples
- `ResourceFieldState` and deterministic region-ready aggregation
- weighted aggregate quality/accessibility
- ocean-resource validation contract
- Stage 2.4 architecture and calibration-status documentation

## Remaining before PASS

- expose resource model through `simulation-model` public API
- add authoritative resource field to `WorldState`
- deterministic seed/geography resource generator
- energy distribution
- metal distribution
- construction/timber distribution
- resource quality/accessibility generation
- save format migration and exact round-trip
- canonical digest integration
- WASM/RenderSnapshot inspection substrate as needed
- same-seed/different-seed tests
- representative-seed resource balance tests
- strict Clippy / locked Rust tests
- headless save/load/replay validation
- permanent source-contract validation
- final evidence and PR merge

## WBS status

- [ ] 2.4.1 food production base placement
- [ ] 2.4.2 energy resource placement
- [ ] 2.4.3 metal resource placement
- [ ] 2.4.4 timber/construction resource placement
- [~] 2.4.5 reserve quantity and quality schema
- [~] 2.4.6 extractable quantity and depletion structure
- [ ] 2.4.7 resource-distribution balance validation

No Stage 2.4 item is marked complete until implementation, integration, validation and evidence all pass.
