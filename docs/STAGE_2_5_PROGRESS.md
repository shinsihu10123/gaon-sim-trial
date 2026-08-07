# Stage 2.5 Dynamic Entity Architecture

Status: **IN PROGRESS**

WBS v2.0 scope:

- 2.5.1 Stable Entity ID rules
- 2.5.2 common Entity Registry interface
- 2.5.3 Country Registry
- 2.5.4 Region Registry dynamicization
- 2.5.5 City Registry foundation
- 2.5.6 Entity Create / Remove lifecycle
- 2.5.7 Entity Lookup / Iteration
- 2.5.8 Reference Integrity
- 2.5.9 deterministic creation order and ID issuance
- 2.5.10 dynamic Entity Save / Load
- 2.5.11 dynamic Entity canonical hash / replay
- 2.5.12 Entity create/remove Event Ledger facts

Implementation direction:

- stable typed u64 entity IDs; zero is invalid
- IDs are monotonic and never recycled after removal
- deterministic BTreeMap-backed registries
- system lifecycle commands share the canonical command journal
- authoritative save format includes registry allocator state
- JS RenderSnapshot boundary uses decimal-string IDs to avoid IEEE-754 precision loss

Current integration gate:

- model registries and Region Registry are present
- Region removal is atomic and surviving IDs are not renumbered
- lifecycle execution is routed through deterministic System Commands
- Save v5 migration includes Registry allocator state and u64 IDs
- lifecycle facts are represented in Event Ledger payloads
- RenderSnapshot crosses the JS boundary with decimal-string entity IDs
- rustfmt and model-level strict Clippy findings are resolved
- Save validation recognizes System command attribution
- Save registry iteration/import findings are corrected
- permanent Core/Viewer CI is proceeding into binary codec, tests and headless replay validation

No item is complete until permanent Core and Viewer validation and evidence are recorded.
