# Stage 1.2 Evidence Record

## Scope

Stage 1.2 certifies the simulation time system and fixed daily tick already used by the Stage 1 core.

The authoritative time rule is:

```text
1 simulation tick = 1 simulated day
```

Playback speed changes how many daily ticks are processed per real second, not the meaning or ordering of a tick.

## Implemented requirements

- simulation start date: Year 1, Month 1, Day 1
- deterministic 365-day calendar without leap years
- exact one-day date advancement
- month, quarter and year boundary reporting
- pause
- 1x / 7x / 30x / 90x / 365x playback modes
- manual one-day step
- manual 30-day step
- integer nanosecond real-time accumulator
- high-speed render sampling independent from daily simulation logic
- playback-speed independence of deterministic state

## Calendar validation

`SimulationDate` tests verify:

- Year 1 / Month 1 / Day 1 stable start
- valid/invalid calendar dates
- January to February rollover
- quarter rollover
- year rollover
- ordinal day values
- quarter number values

The simulation calendar does not use host locale, timezone or system time.

## Tick and speed validation

`simulation-core` tests verify:

- one `tick()` advances exactly one simulated day
- `step_one_month()` executes exactly 30 daily ticks
- paused mode executes zero ticks
- 1x, 7x, 30x, 90x and 365x map to the planned daily tick counts
- ten 100 ms updates at 7x accumulate to exactly seven simulated days
- 90x and 365x reduce render sampling without skipping simulation ticks
- 365 manual daily ticks and one real second at 365x produce the same deterministic result

Stage 1.6 additionally verifies exact authoritative snapshot and canonical digest equality between manual and 365x playback when commands and events are present.

## Logic/render separation

The simulation processes every daily tick at high speed. `render_stride_days()` only determines how often presentation snapshots need to be sampled:

- 30x and below: every day
- 90x: every 3 simulated days
- 365x: every 13 simulated days

Renderer sampling is not authoritative state.

## GitHub validation

Authoritative validation was executed by GitHub Actions `Stage 1 Core CI` on pull request #5.

Passing run:

- workflow run id: `31160742588`
- validated feature head: `49fc62d23b372fd51b52801274a1d8f66a4cdd58`
- pull-request merge test commit: `8a175a27f27a6cc389c2d4a20cd238b424470b54`
- Rust: `1.97.1`
- dependency resolution: committed `Cargo.lock` with `--locked`

Passed gates:

1. `cargo fmt --all -- --check`
2. `cargo clippy --locked --workspace --all-targets -- -D warnings`
3. `cargo test --locked --workspace`
4. `cargo run --locked -p simulation-runner --quiet`
5. Stage 1.2 time/tick contract
6. Stage 1.3 command contract
7. Stage 1.4 Event Ledger contract
8. Stage 1.5 save/restore contract
9. Stage 1.6 determinism contract

## Test result

- `simulation-core`: **26 passed, 0 failed**
- `simulation-model`: **7 passed, 0 failed**
- `simulation-save`: **6 passed, 0 failed**
- total Rust unit tests: **39 passed, 0 failed**

## Headless regression result

```text
date=0002-01-01 elapsed_days=365 ticks=365 commands=2 events=2 journal=2 render_stride=13 save_json_bytes=249 save_state_bytes=218 canonical_digest=5c4df237597b9f79 random_probe=6f6ec8e35d96b3d5
```

This confirms that the certified time system remains compatible with command processing, Event Ledger, save/restore, canonical digest, deterministic random generation and command-journal replay.

## Decision

Status: **PASS**

Stage 1.2 satisfies implementation + locked automated validation + recorded evidence. Future monthly, quarterly, yearly and event-driven domain systems must consume these calendar boundaries without changing the one-tick/one-day contract.
