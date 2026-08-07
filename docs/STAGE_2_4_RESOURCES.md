# Stage 2.4 — Deterministic Resource Generation

## Purpose

Stage 2.4 creates spatial resource endowments from the world seed and geography without assigning economic roles to countries in advance.

Resources are first generated on the canonical 129×129 terrain substrate. When the 60 actual regions are spatially assigned later in Stage 2, the same cells are aggregated into region-level endowments. No national resource total is generated directly.

## Resource families

### Food production base

Food is a renewable geography-dependent annual production base rather than a finite mineral stock.

Unit: tonnes/year per canonical 100 km² terrain cell before labor, technology, capital, irrigation and agricultural infrastructure.

The base is derived from Stage 2.3 agricultural suitability.

### Energy

Finite reserve.

TEST unit: GWh-equivalent accounting units.

This aggregates energy-resource potential into one strategic resource family for the TEST build. Detailed fuel types are deferred.

### Metals

Finite reserve.

TEST unit: kilotonne-equivalent accounting units.

Detailed iron/copper/rare-metal separation is deferred.

### Timber and construction resources

Finite strategic construction-resource stock.

TEST unit: kilotonne-equivalent accounting units.

The TEST abstraction combines timber and basic construction minerals into the resource family specified by the project plan; later versions can split it.

## Finite deposit state

Every finite deposit stores:

- initial quantity
- remaining quantity
- quality permille
- accessibility permille

Quality/accessibility range: `0..=1000`.

A zero-quantity deposit has zero metadata. Remaining quantity may never exceed initial quantity.

## Depletion

`ResourceDeposit::extract(requested_quantity)` removes only the quantity that remains.

The resulting depletion ratio is deterministic:

`depletion_permille = (initial - remaining) × 1000 / initial`

No negative stock or over-extraction is possible.

Actual industrial extraction rate, labor demand, capital requirement and technology effects belong to the later industry system. Stage 2.4 defines the geographically available stock and depletion mechanism only.

## Spatial generation principles

Resource placement must satisfy all of the following:

1. depends on world seed and geography, never country identity;
2. uses independent deterministic resource noise fields;
3. preserves geographic clustering rather than uniform random scattering;
4. food potential is strongly connected to Stage 2.3 agricultural suitability;
5. metal potential may correlate with relief/geological proxy fields;
6. energy potential uses independent clustered fields with terrain-accessibility effects;
7. construction/timber potential uses vegetation and landform inputs;
8. resource quality and accessibility are separate from reserve quantity;
9. ocean samples contain no land-resource stock in the TEST build;
10. same seed and terrain reproduce the same initial resource field.

## Region aggregation

`ResourceFieldState::aggregate_indices()` sums quantities for canonical terrain cells belonging to a later region.

Quality and accessibility are weighted by remaining quantity rather than simple arithmetic mean. This allows a region with one large high-quality deposit and many tiny deposits to retain an economically meaningful aggregate.

## Persistence requirement

Unlike hydrology and landmass IDs, finite resource stocks become dynamic after extraction. Therefore the resource field must become authoritative `WorldState` and be included in save/load and state digest before Stage 2.4 can PASS.

This is the key integration task still remaining in Stage 2.4.

## Calibration status

Resource quantities and distribution constants are TEST calibration assumptions. They are not claimed to represent real global geological reserve estimates.

The architecture separates units, quantity, quality and accessibility so later empirical calibration can replace generation parameters without changing simulation interfaces.

## Acceptance criteria

Stage 2.4 is PASS only when:

1. food-production base is generated for land cells;
2. energy reserves are spatially generated;
3. metal reserves are spatially generated;
4. timber/construction reserves are spatially generated;
5. quantity, quality and accessibility are represented separately;
6. extraction cannot exceed remaining stock;
7. depletion is persisted through save/load;
8. resource state participates in canonical digest;
9. same seed reproduces initial distribution;
10. different seeds produce different distributions;
11. no resource family is assigned according to future country identity;
12. representative seeds pass distribution-balance tests;
13. region-ready aggregation works deterministically;
14. locked Rust, save/replay and Viewer contracts remain green.
