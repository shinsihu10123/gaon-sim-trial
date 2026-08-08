# Stage 2.6 Initial Human Groups and Ecological Placement

Status: **PASS**

Authoritative scope: WBS v3.1 civilization-origin revision.

## WBS v3.1 completion

- ■ 2.6.1 starting HumanGroup count non-hardcoded generation interface
- ■ 2.6.2 initial total population / per-group population parameter structure
- ■ 2.6.3 habitable Region based HumanGroup placement
- ■ 2.6.4 initial food / basic resource state
- ■ 2.6.5 initial mobility / behavior traits
- ■ 2.6.6 initial Knowledge Profile structure
- ■ 2.6.7 regional carrying-capacity / natural-food linkage
- ■ 2.6.8 Year-1 Country = 0 verification
- ■ 2.6.9 modern border/capital/tax/GDP inactivity verification
- ■ 2.6.10 initial inter-group distance / contact possibility
- ■ 2.6.11 seed reproducibility
- ■ 2.6.12 initial HumanGroup RenderSnapshot output

## Architecture fixed by Stage 2.6

- Year 1 civilization-origin initialization is parameter driven; no canonical HumanGroup count or total population is hardcoded in the generator.
- Initial HumanGroups are placed only on eligible land Regions with non-zero natural carrying capacity and geography-derived food capacity.
- Habitat ranking uses geography and resources plus deterministic seed-derived tie-breaking, never future national roles or scripted historical outcomes.
- Configured total population is conserved exactly while group populations may differ deterministically.
- Initial food stock, basic-resource stock, mobility, exploration and settlement-bias state are authoritative HumanGroup seed state.
- Initial Knowledge is an extensible profile container; Stage 2.6 does not freeze the final Knowledge taxonomy or Technology Space.
- Pairwise contact possibility is derived from deterministic integer geographic distance and an explicit scenario radius.
- Initial HumanGroup creation leaves Country, City, Settlement, Community and PoliticalEntity registries empty and leaves Region ownership/control unclaimed.
- Modern borders, capitals, GDP, taxation and fiscal systems therefore remain inactive in this stage.
- Save format v6 serializes initialized HumanGroup ecological state and allocator state.
- RenderSnapshot v5 exports initialized HumanGroups with JavaScript-safe decimal-string entity IDs.
- HumanGroup 3D marker rendering is deliberately deferred; Stage 2.6 exposes authoritative render data only.

## Acceptance coverage

World-generation acceptance verifies:

- variable configured group counts
- exact total-population conservation
- distinct eligible starting Regions
- valid initial food/basic-resource stocks
- configured behavior trait bounds
- zero-Country and zero-modern-state pre-state initialization
- unclaimed Region ownership/control
- same-seed reproduction and different-seed variation
- complete pairwise contact-distance coverage

Persistence acceptance verifies:

- Save v6 initialized HumanGroup round-trip equality
- authoritative HumanGroup state changes canonical binary/checksum input

Render acceptance verifies:

- zero-Country world can expose initialized HumanGroups through RenderSnapshot v5
- HumanGroup and Region identifiers cross the JS boundary as decimal strings

Permanent source contracts verify the Stage 2.6 model, generator, persistence, RenderSnapshot, TypeScript and acceptance-test invariants and are chained into Viewer CI.

## Clean functional evidence head

Commit: `a6a966391ca8f00d9ab03918a7431192529e2995`

Permanent CI:

- Core run `31236505099`: PASS
  - rustfmt PASS
  - strict Clippy PASS
  - complete Rust workspace tests PASS
  - headless save/load/replay PASS
  - Stage 1 command contract PASS
- Viewer run `31236505114`: PASS
  - locked Rust workspace PASS
  - actual WASM build PASS
  - npm dependency installation PASS
  - TypeScript typecheck PASS
  - Vite production build PASS
  - Stage 1.1 / 2.1 / 2.2 / 2.3 / 2.4 / 2.5 / 2.6 permanent contract chain PASS

Rust test total on the clean functional evidence head: **109 passed / 0 failed**.

Stage 2.6 direct acceptance tests on that head:

- 5 initial HumanGroup world-generation tests PASS
- 2 Save v6 HumanGroup tests PASS
- 1 RenderSnapshot v5 HumanGroup test PASS

Headless evidence:

- date: `0002-01-01`
- elapsed days: `365`
- terrain samples: `16641`
- land samples: `7371`
- river samples: `2145`
- drainage basins: `409`
- islands: `1`
- save state bytes: `1165241`
- canonical digest: `e965a88a5ec817bf`

## Cleanup evidence

Development-only Stage 2.6 migration tooling was removed before the functional evidence head. PR #14 contains only product/source changes, acceptance tests, permanent contracts and documentation; no Stage 2.6 one-shot migration workflow or migration/fixer script remains.

## Final completion rule

This PASS document is committed only after the cleaned functional head passed both permanent Core and Viewer gates. The evidence-document commit must itself pass the same permanent Core and Viewer CI before PR #14 is merged into `develop`.

Stage 2.7 is not started by this document.
