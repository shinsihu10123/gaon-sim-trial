# Stage 2.5 Dynamic Entity Architecture — Civilization-Origin Generalization

Status: **IN PROGRESS — FINAL VALIDATION**

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

## Implemented foundation

- Region count is dynamic; 60 is Benchmark S only.
- sparse Region IDs survive removal without renumbering.
- HumanGroup / Settlement / Community / PoliticalEntity / Country / City typed registries exist.
- Region Registry is dynamic and preserves stable IDs.
- System lifecycle commands create and remove pre-state entities.
- `PromotePoliticalEntityToCountry` provides a deterministic transition mechanism without predefining the historical cause of state emergence.
- Country / City / Region reference-integrity checks remain in the common lifecycle layer.
- Save format v5 serializes all current dynamic registries and allocator cursors.
- command binary codec includes all pre-state lifecycle commands and political-entity promotion.
- Event Ledger records create, remove, rejected mutation and political transition facts.
- RenderSnapshot crosses the JS boundary with decimal-string IDs where entity identifiers are exposed.

## Acceptance coverage

- zero-country world can contain HumanGroup, Settlement, Community and PoliticalEntity entities
- monotonic IDs are not recycled after removal
- lifecycle payloads require System source
- referenced entities are protected from invalid removal
- all pre-state registry allocators survive Save / Load
- PoliticalEntity can transition to Country without embedding emergence criteria in Stage 2.5
- civilization-origin lifecycle Command Journal replay matches authoritative digest
- sparse Region identity and allocator survive Save / Load

## Current gate

The pre-v3.1 Stage 2.5 Core gate was restored to PASS before civilization-origin expansion. The generalized v3.1 implementation is now in permanent Core / Viewer CI revalidation with a dedicated Stage 2.5 source contract.

No Stage 2.5 item is marked complete until the final civilization-origin head passes permanent Core and Viewer CI and evidence is recorded.
