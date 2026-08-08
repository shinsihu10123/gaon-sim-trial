# Stage 2.6 Initial Human Groups and Ecological Placement

Status: **IN PROGRESS**

Authoritative scope: WBS v3.1 civilization-origin revision.

## WBS v3.1 scope

- ◐ 2.6.1 starting HumanGroup count non-hardcoded generation interface
- ◐ 2.6.2 initial total population / per-group population parameter structure
- ◐ 2.6.3 habitable Region based HumanGroup placement
- ◐ 2.6.4 initial food / basic resource state
- ◐ 2.6.5 initial mobility / behavior traits
- ◐ 2.6.6 initial Knowledge Profile structure
- ◐ 2.6.7 regional carrying-capacity / natural-food linkage
- ◐ 2.6.8 Year-1 Country = 0 verification
- ◐ 2.6.9 modern border/capital/tax/GDP inactivity verification
- ◐ 2.6.10 initial inter-group distance / contact possibility
- ◐ 2.6.11 seed reproducibility
- ◐ 2.6.12 initial HumanGroup RenderSnapshot output

No item is complete until implementation, permanent Core/Viewer validation and evidence all exist.

## Current implementation

- parameterized `InitialHumanGroupConfig`
- deterministic habitat ranking from land Region + terrain carrying capacity + natural food capacity
- exact total-population distribution across variable group counts
- deterministic mobility / exploration / settlement-bias seed traits
- food person-day and basic-resource initialization
- extensible Knowledge seed-profile container without fixing the final taxonomy
- pairwise integer distance and contact-radius classification
- dedicated worldgen acceptance tests
- Save v6 migration design and round-trip tests
- RenderSnapshot v5 migration design and zero-country HumanGroup snapshot test
- permanent Stage 2.6 source contract chained into Viewer CI

## Explicit non-goals

- no fixed starting HumanGroup count
- no fixed starting total population
- no Country creation
- no modern border/capital/GDP/tax activation
- no state-emergence causal model
- no settlement formation yet
- no HumanGroup 3D marker yet

## Current integration gate

The source migration that extends `HumanGroupEntity`, Save v6 and RenderSnapshot v5 has been prepared but is not yet applied to the branch. The next gate applies that migration, runs rustfmt, opens the Stage 2.6 draft PR and uses permanent Core/Viewer CI to identify real integration failures.
