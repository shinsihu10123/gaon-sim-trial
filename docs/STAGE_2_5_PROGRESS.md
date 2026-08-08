# Stage 2.5 Dynamic Entity Architecture — Civilization-Origin Generalization

Status: **IN PROGRESS**

Authoritative scope: WBS v3.1 civilization-origin revision.

## WBS v3.1 scope

- 2.5.1 Stable Entity ID rules
- 2.5.2 common Entity Registry interface
- 2.5.3 Region Registry dynamicization
- 2.5.4 HumanGroup Registry
- 2.5.5 Settlement / Community Registry
- 2.5.6 PoliticalEntity / Country Registry
- 2.5.7 Entity Create / Remove lifecycle
- 2.5.8 Lookup / Iteration / Reference Integrity
- 2.5.9 deterministic creation order and ID issuance
- 2.5.10 dynamic Entity Save / Load
- 2.5.11 dynamic Entity canonical hash / replay
- 2.5.12 lifecycle / transition Event Ledger facts

## Architecture rules

- Year 1 may legitimately contain zero countries.
- Country is not an initial-world invariant; it is a later political entity type.
- stable typed u64 entity IDs; zero is invalid
- IDs are monotonic and never recycled after removal
- deterministic BTreeMap-backed registries
- System Commands own authoritative lifecycle mutations
- registry allocator state is part of canonical persistence
- command-journal replay must reproduce allocator and entity state
- JS RenderSnapshot boundary uses decimal-string IDs to avoid IEEE-754 precision loss

## Existing validated foundation retained

- Region count is dynamic; 60 is Benchmark S only.
- sparse Region IDs survive removal without renumbering.
- Country / Region / City registry groundwork exists.
- Region reciprocal adjacency maintenance exists.
- Country / City reference-integrity checks exist.
- Save format v5 is the unmerged Stage 2.5 schema and can still be extended before PASS.
- Entity lifecycle facts use the Event Ledger.

## Civilization-origin generalization in progress

- HumanGroup typed ID and registry
- Settlement typed ID and registry
- Community typed ID and registry
- PoliticalEntity typed ID and registry
- authoritative WorldState ownership of all pre-state registries
- System lifecycle commands for each pre-state entity type
- binary persistence and command-journal codec extension
- acceptance tests for zero-country initial state, non-reused IDs, save/load and replay

## Current gate

The pre-v3.1 Stage 2.5 Core gate was restored to PASS before civilization-origin expansion. The branch is now intentionally open again while the new v3.1 entity set is integrated and revalidated.

No Stage 2.5 item is marked complete until the final civilization-origin head passes permanent Core and Viewer CI and evidence is recorded.
