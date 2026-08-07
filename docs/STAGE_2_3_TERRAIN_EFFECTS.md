# Stage 2.3 — Terrain Simulation Effects

## Purpose

Stage 2.3 converts Stage 2.2 geography into deterministic inputs that later economic, infrastructure, population and military systems can consume directly.

The terrain layer does not decide economic outcomes by itself. It provides geography-only coefficients; technology, infrastructure, institutions, capital and policy are applied in later stages.

## Numeric convention

Most effects use integer permille fixed point:

- `1000` = neutral 1.00×
- `1250` = 1.25×
- `800` = 0.80×

This avoids authoritative floating-point state and gives downstream formulas explicit physical/economic meaning.

Natural carrying capacity is expressed as **people per km²**, before urban infrastructure and modern technology.

Ocean samples return zero land effects. Maritime movement is modeled separately in the transport system rather than pretending ocean is land terrain.

## 2.3.1 Agriculture suitability

Field: `agriculture_yield_permille`

Inputs:
- biome
- relief
- moisture relative to a temperate optimum
- river order

Interpretation:

`terrain-adjusted agricultural output = base agricultural output × agriculture_yield_permille / 1000`

Grassland/plains with adequate moisture are favored; desert, alpine and mountain terrain are penalized; river access provides a limited positive contribution.

Range in the TEST model: `0..=1400`.

## 2.3.2 Construction difficulty

Field: `construction_cost_permille`

Interpretation:

`terrain-adjusted construction cost = base construction cost × construction_cost_permille / 1000`

Plains are the neutral reference. Hills, mountains, forest, wetland and alpine cover increase cost. Coast has a small baseline premium reflecting stabilization and coastal works.

Land range in the TEST model: `1000..=2200`.

## 2.3.3 Movement resistance

Field: `movement_cost_permille`

Interpretation:

`terrain-adjusted movement cost/time = base route cost/time × movement_cost_permille / 1000`

Relief, vegetation/wetland cover and strategic-scale river crossings contribute. Roads and railways will later reduce the effective cost in Stage 4 rather than modifying the underlying terrain coefficient.

Land range in the TEST model: `1000..=2200`.

## 2.3.4 Defense modifier

Field: `defense_multiplier_permille`

Interpretation:

`terrain-adjusted defensive effectiveness = base defensive effectiveness × defense_multiplier_permille / 1000`

Open plains are below neutral; hills, mountains, forest and wetlands provide greater defensive advantage. Fortifications and military quality are separate later multipliers.

Land range in the TEST model: `850..=1700`.

## 2.3.5 Port feasibility

Field: `port_feasibility_permille`

Non-coastal land is `0`.

Coastal feasibility is influenced by:
- coastal classification
- low elevation
- whether the cell is a derived river mouth
- wetland penalty

Range: `0..=1000`.

This value indicates whether a coastal sample is geographically suitable for a port. Actual port construction still requires state investment and resources in Stage 4.

## 2.3.6 Natural carrying capacity

Field: `carrying_capacity_people_per_km2`

The terrain contribution combines:
- agricultural potential
- coast access
- river order
- high-altitude penalty

TEST range for land: `5..=280 people/km²`.

This is not a hard population ceiling for modern cities. Infrastructure, sanitation, food imports, technology and services can raise effective urban capacity in later stages.

## 2.3.7 Regional productivity correction

Field: `productivity_multiplier_permille`

This is a geography-only accessibility/productive-location modifier derived primarily from:
- construction burden
- movement burden
- coast access
- river access

Interpretation:

`geography-adjusted base productivity = base productivity × productivity_multiplier_permille / 1000`

TEST range: `600..=1150`.

It must not replace technology, education, capital, institutions or infrastructure. Those systems multiply or otherwise modify this geographic base later.

## Region aggregation

`TerrainEffectField::aggregate_indices()` averages canonical terrain samples belonging to a region.

Stage 2.3 therefore does not need to invent national scores. Once later Stage 2 work assigns canonical heightfield samples to the 60 regions, the same effect field can be aggregated directly into region-level geography inputs.

## Determinism

Terrain effects are derived from:
- canonical TerrainState
- deterministic hydrology derived from that terrain

They are not independently serialized. Same terrain state therefore always yields the same effect field, and save/load/replay cannot produce a second conflicting geography-effect state.

## Calibration status

The numerical coefficients in Stage 2.3 are **TEST model calibration assumptions**, not claimed empirical elasticities.

Their purpose is to establish correct causal direction, unit conventions, bounded behavior and system connectivity. Empirical recalibration can later replace constants without changing the architecture.

## Acceptance criteria

Stage 2.3 is PASS only when:

1. agriculture suitability varies deterministically with terrain, moisture and rivers;
2. construction cost is lower on accessible flat land than difficult mountain/wetland terrain;
3. movement resistance varies with relief, cover and river crossings;
4. mountain/hill terrain provides greater defense than open plains;
5. only coastal cells have non-zero port feasibility;
6. carrying capacity is positive and geographically differentiated on land;
7. productivity modifier is bounded and geographically differentiated;
8. ocean samples expose no land effects;
9. same seed produces byte/logically identical effect fields;
10. effect arrays align one-to-one with canonical terrain samples;
11. region-ready aggregation is deterministic;
12. locked Rust tests, strict Clippy and permanent Viewer source-contract validation pass.
