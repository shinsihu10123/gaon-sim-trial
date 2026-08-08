# Trial Archive Index

Archive date: 2026-08-09
Repository: `shinsihu10123/gaon-sim-trial`
Purpose: preserve the Gaon simulation trial implementation and development evidence without mixing it with production development.

## Canonical preserved states

### A. Final validated trial baseline

- Branch: `archive/trial-stable-stage-3.1`
- Commit: `2c6247da0d8c47f4d8e8ddf5a5771da79bf0d191`
- Source: merged PR #19
- Status: validated trial baseline
- Meaning: Stage 3.1 authoritative HumanGroup state persistence is the final trial slice accepted as stable.

### B. Final unfinished trial experiment

- Branch: `archive/trial-stage-3.2-wip`
- Commit: `a03cc252f32787407c8128cca3283e8c001bb2cb`
- Source: PR #20
- Status: WIP / not production-ready
- CI at archive point: Viewer CI PASS, Core CI FAIL
- Meaning: Stage 3.2 population and survival implementation is preserved for reference but is not part of the validated baseline.

## Development-line policy

- `main`: archive landing and repository notice.
- `develop`: historical trial integration line. No new production features should be developed here.
- `feature/*` and `agent/*`: historical implementation branches are intentionally retained. Do not delete them as part of the trial archive.
- `archive/*`: explicit immutable reference points created when the project moved from trial to production planning.

## Major trial milestones retained

1. Deterministic fixed-tick Rust simulation foundation.
2. Command/Event/Save/Replay and deterministic state verification.
3. Dynamic world Region architecture.
4. Terrain, hydrology/effects, and resource generation foundations.
5. Dynamic Stable Entity architecture with zero-country civilization-origin support.
6. Initial HumanGroup generation and ecological placement.
7. Viewer camera, selection, WASM bridge, and dynamic 3D terrain/HumanGroup presentation.
8. Authoritative HumanGroup runtime state with Save Format v7 / RenderSnapshot v7.
9. Stage 3.2 population/survival experiment preserved separately as WIP.

## Production handoff rule

Nothing in this repository is automatically designated as production architecture merely because it worked in the trial. When production development starts, each subsystem must be reviewed and classified as one of:

- `KEEP` — reuse with validation;
- `EXTEND` — retain architecture but expand capability;
- `REWORK` — reuse concepts/interfaces but redesign implementation;
- `REPLACE` — do not carry the trial implementation into production.

The production repository/development line should have a distinct name and must not reuse `gaon-sim-trial` as its canonical repository identifier.

## PR #20 preservation note

PR #20 is intentionally kept open as a Draft and renamed with `[TRIAL ARCHIVE / WIP]`. This preserves its discussion, diff, CI evidence, and original feature branch without implying that the code was accepted.
