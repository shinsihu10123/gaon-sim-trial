# Stage 2.5 Dynamic Entity Architecture — Civilization-Origin Generalization

Status: **PASS**

Authoritative scope: WBS v3.1 civilization-origin revision.

## WBS v3.1 completion

- ■ 2.5.1 Stable Entity ID rules
- ■ 2.5.2 common Entity Registry interface
- ■ 2.5.3 Region Registry dynamicization
- ■ 2.5.4 HumanGroup Registry
- ■ 2.5.5 Settlement / Community Registry
- ■ 2.5.6 PoliticalEntity / Country Registry
- ■ 2.5.7 Entity Create / Remove lifecycle
- ■ 2.5.8 Lookup / Iteration / Reference Integrity
- ■ 2.5.9 deterministic creation order and ID issuance
- ■ 2.5.10 dynamic Entity Save / Load
- ■ 2.5.11 dynamic Entity canonical hash / replay
- ■ 2.5.12 lifecycle / transition Event Ledger facts

## Architecture rules fixed by Stage 2.5

- Year 1 may legitimately contain zero countries.
- Country is not an initial-world invariant; it is a later political entity type.
- Stable typed u64 entity IDs are non-zero, monotonic and never recycled.
- Dynamic registries use deterministic BTreeMap ordering.
- WorldState authoritatively owns HumanGroup, Settlement, Community, PoliticalEntity, Country, City and Region registries.
- System Commands own authoritative lifecycle mutations.
- `PromotePoliticalEntityToCountry` supplies the transition mechanism without embedding the historical cause of state emergence in Stage 2.5.
- Registry allocator state is part of canonical persistence.
- Command Journal replay reproduces entity state, transition events and allocator state.
- RenderSnapshot uses decimal-string entity IDs at the JavaScript boundary to avoid IEEE-754 precision loss.

## Implemented foundation

- Region count is dynamic; 60 is Benchmark S only.
- Sparse Region IDs survive removal without renumbering.
- HumanGroup / Settlement / Community / PoliticalEntity / Country / City typed registries are implemented.
- Region Registry is dynamic and preserves stable IDs.
- System lifecycle commands create and remove pre-state entities.
- Country / City / Region reference-integrity checks remain in the common lifecycle layer.
- Save format v5 serializes all current dynamic registries and allocator cursors.
- Command binary codec includes all pre-state lifecycle commands and PoliticalEntity -> Country promotion.
- Event Ledger records create, remove, rejected mutation and political transition facts.
- Permanent `check-dynamic-entity-contract.mjs` is chained into Viewer CI.

## Acceptance coverage

- zero-country world can contain HumanGroup, Settlement, Community and PoliticalEntity entities
- monotonic IDs are not recycled after removal
- lifecycle payloads require System source
- referenced entities are protected from invalid removal
- all pre-state registry allocators survive Save / Load
- PoliticalEntity can transition to Country without predefining emergence criteria
- civilization-origin lifecycle Command Journal replay matches authoritative digest
- sparse Region identity and allocator survive Save / Load

## Functional evidence head

Commit: `accee48a670990deedbec8f6d675e9e9cea105a3`

Permanent CI on that cleaned feature head:

- Core run `31233701411`: PASS
  - rustfmt PASS
  - strict Clippy PASS
  - complete Rust workspace tests PASS
  - headless save/load/replay PASS
  - Stage 1 command contract PASS
- Viewer run `31233701400`: PASS
  - locked Rust workspace PASS
  - actual WASM build PASS
  - npm dependency installation PASS
  - TypeScript typecheck PASS
  - Vite production build PASS
  - Stage 1.1 / 2.1 / 2.2 / 2.3 / 2.4 / 2.5 source contract chain PASS

Rust test total on the functional evidence head: **101 passed / 0 failed**.

Headless evidence:

- date: `0002-01-01`
- elapsed days: `365`
- terrain samples: `16641`
- land samples: `7371`
- river samples: `2145`
- drainage basins: `409`
- islands: `1`
- save state bytes: `1165241`
- canonical digest: `c796422d1225847e`

## Final completion rule

This PASS document is itself committed after the cleaned functional head passed both permanent gates. The documentation/evidence commit must also pass the same permanent Core and Viewer CI before PR #13 is merged into `develop`.

Stage 2.6 is not started by this document.
