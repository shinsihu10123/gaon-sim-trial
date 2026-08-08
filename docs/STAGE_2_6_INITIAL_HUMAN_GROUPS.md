# Stage 2.6 Initial Human Groups and Ecological Placement

## Purpose

Stage 2.6 initializes the civilization-origin world with HumanGroup entities before any Country exists.

It does **not** create modern states, borders, capitals, GDP, taxation or fiscal systems. Those remain inactive until later emergence and state-system stages.

## Parameter-driven initialization

The core generator accepts an explicit `InitialHumanGroupConfig` rather than defining a fixed starting population or fixed number of groups.

Scenario parameters include:

- HumanGroup count
- total population
- population-size variation range
- initial food-stock day range
- basic resource units per person
- mobility range
- exploration range
- settlement-bias range
- contact radius
- initial Knowledge profile

The exact canonical scenario values remain a later scenario-design decision.

## Geography-first placement

HumanGroup placement consumes existing authoritative world data:

1. dynamic Region registry
2. canonical terrain
3. Stage 2.3 terrain-effect field
4. Stage 2.4 resource field

Only land Regions with non-zero natural carrying capacity and non-zero geography-derived food capacity are eligible.

Candidate ranking uses normalized natural food capacity and carrying capacity plus a deterministic seed-derived tie-breaker. Country role, future industry, future political power and future civilization outcome are not inputs.

Each Year-1 HumanGroup is initially assigned to a distinct eligible Region. This is a trial initialization constraint, not a permanent hard cap on later population density or group count.

## Initial HumanGroup state

Stage 2.6 adds authoritative seed state to `HumanGroupEntity`:

- Region identity
- x/z location
- canonical terrain sample index
- population
- food stock in person-days
- basic-resource stock units
- mobility permille
- exploration permille
- settlement-bias permille
- extensible initial Knowledge profile

Knowledge is represented as sorted `(domain_key, level_permille)` seed values. Stage 2.6 does not define the final Knowledge taxonomy or Technology Space.

## Population allocation

The configured total population is conserved exactly.

Per-group population variation is generated deterministically within the configured variation envelope. No group receives zero population.

## Initial stocks

Food stock is expressed as person-days and derived from:

- group population
- explicit configured food-days range
- local natural food accessibility

Basic resources are derived from:

- group population
- explicit per-person resource parameter
- local construction-resource accessibility

These are initialization stocks only. Full survival production/consumption belongs to Stage 3.

## Behavior seeds

Mobility, exploration and settlement bias are deterministic seed-derived values constrained by explicit configuration ranges.

They are initial heterogeneity inputs, not predetermined historical roles.

## Contact possibility

Initial pairwise contact information is derived from integer geographic distance and an explicit contact-radius parameter.

The calculation provides only geometric contact possibility. Trade, conflict, diplomacy, culture and state relations are not created here.

## Pre-state invariant

After Stage 2.6 initialization:

- HumanGroup entities may exist.
- Country, City, Settlement, Community and PoliticalEntity registries remain empty.
- Region legal ownership and control remain unclaimed.
- no modern state boundary, capital, GDP, tax or fiscal system is activated by the initializer.

This invariant has dedicated acceptance coverage and is also required by the permanent Stage 2.6 source contract.

## Determinism

Same world data + same configuration + same seed must produce identical:

- HumanGroup IDs
- locations
- populations
- stocks
- behavior seeds
- contact distances

Changing the seed may alter placement ordering, population allocation and behavior values without changing the explicit scenario totals.

## Persistence

HumanGroup initialization becomes authoritative mutable world state and is therefore serialized directly.

Stage 2.6 advances the save format to v6. Save/Load must preserve:

- HumanGroup stable IDs
- allocator cursor
- all initialized ecological state
- Knowledge seed profile

Canonical binary and digest must change when authoritative HumanGroup seed state changes.

## RenderSnapshot

Stage 2.6 advances RenderSnapshot to v5 and exports initialized HumanGroups as dynamic records with JS-safe string IDs.

This stage exposes data only. Actual HumanGroup marker rendering belongs to Stage 2.8.

## Non-goals

Stage 2.6 does not define:

- a fixed number of starting HumanGroups
- a fixed total starting population
- modern countries
- modern borders
- capitals or cities
- tax/GDP/fiscal systems
- state emergence criteria
- settlement formation rules
- survival simulation
- migration simulation
- culture formation
- final Knowledge taxonomy
