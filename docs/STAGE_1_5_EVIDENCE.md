# Stage 1.5 Evidence Record

## Scope

Stage 1.5 establishes exact versioned save/restore for every authoritative structure that exists in the Stage 1 simulation core.

A save is valid only when restoration preserves both the current state and the future deterministic trajectory.

## WBS coverage

- `1.5.1` Save format definition — COMPLETE
- `1.5.2` JSON metadata — COMPLETE
- `1.5.3` Binary authoritative state — COMPLETE
- `1.5.4` Manual save path — COMPLETE
- `1.5.5` Autosave path/cadence contract — COMPLETE
- `1.5.6` Load and exact engine reconstruction — COMPLETE
- `1.5.7` Save-format version validation — COMPLETE
- `1.5.8` Corruption and malformed-state rejection — COMPLETE
- `1.5.9` Exact state comparison after restore — COMPLETE
- `1.5.10` Long-run save/restore continuation validation — COMPLETE

## Persisted authoritative state

The Stage 1 save snapshot contains:

- `WorldState`
- pending command queue
- executed command history
- append-only Event Ledger
- next `CommandId`
- next `EventId`

Renderer frame rate, playback speed and wall-clock timing are not authoritative simulation state and are intentionally excluded.

## File contract

Logical save representation:

```text
metadata.json
state.bin
```

`metadata.json` records:

- format version
- manual/autosave origin
- simulation date
- elapsed simulated days
- 64-bit seed as fixed-width hexadecimal text
- binary byte length
- FNV-1a 64-bit integrity checksum

`state.bin` uses:

- magic `GAONST01`
- format version `1`
- canonical field order
- little-endian integer encoding
- explicit enum tags
- bounded record counts

## Restore validation

Restore is transactional. A new engine is constructed only after:

1. metadata parses successfully;
2. metadata version is supported;
3. binary length agrees with metadata;
4. checksum matches;
5. binary magic/version/tags/dates decode correctly;
6. no truncation or trailing bytes exist;
7. snapshot cross-record invariants pass;
8. metadata and binary state agree.

Current snapshot invariants verify command identity/order, pending-command dates, command source/priority agreement, execution dates, Event Ledger append order, event references, source attribution, next IDs, and world date/elapsed-day consistency.

## Long-run continuation test

The critical restore test performs:

1. deterministic engine creation with seed `2026`;
2. immediate user command submission;
3. country-AI command scheduled for Year 75;
4. 50 simulated years of execution;
5. autosave creation;
6. restoration into a second engine;
7. exact snapshot equality at the restore point;
8. 50 additional simulated years on both engines;
9. exact snapshot, command-log, Event-Ledger and state-digest equality at Year 101.

The Year-75 command is still pending when the Year-51 save is created. Therefore this test verifies that future scheduled commands survive restoration and execute at the same future point rather than merely proving date restoration.

## GitHub validation

Authoritative validation was executed by GitHub Actions `Stage 1 Core CI` on pull request #3.

Passing run:

- workflow run id: `31159404109`
- validated feature head: `99753b2b347c56a43c5abf7966fe4f8fbe3a2f23`
- pull-request merge test commit: `c0933c2e57affff192082f4ccfdd7ecc9697e5fc`
- Rust: `1.97.1`

Passed gates:

1. `cargo fmt --all -- --check`
2. `cargo clippy --workspace --all-targets -- -D warnings`
3. `cargo test --workspace`
4. `cargo run -p simulation-runner --quiet`
5. Stage 1.3 command contract
6. Stage 1.4 Event Ledger contract
7. Stage 1.5 save/restore contract

## Test result

- `simulation-core`: **20 passed, 0 failed**
- `simulation-model`: **7 passed, 0 failed**
- `simulation-save`: **6 passed, 0 failed**
- `simulation-runner`: 0 unit tests
- total Rust unit tests: **33 passed, 0 failed**

Verified cases include:

- exact metadata/binary round trip
- checksum corruption rejection
- unsupported metadata version rejection
- invalid autosave interval rejection
- autosave cadence behavior
- inconsistent next-ID rejection
- exact pending/executed/event state restoration
- 100-year original/restored continuation equality

## Headless validation

```text
date=0002-01-01 elapsed_days=365 ticks=365 commands=2 events=2 render_stride=13 save_json_bytes=249 save_state_bytes=218 digest=0476ea3ebacedc22
```

The runner executed 365 daily ticks, processed two commands, emitted two events, created a validated save bundle, restored a second engine, asserted exact snapshot equality, and preserved the existing deterministic state digest.

## Dependency note

The CI resolved the newly introduced serialization dependencies successfully. A repository `Cargo.lock` has not yet been committed; dependency locking should be addressed as part of the reproducibility hardening in Stage 1.6 / development-environment maintenance rather than being falsely treated as present here.

## Decision

Status: **PASS**

Stage 1.5 satisfies the project completion rule of implementation + strict automated validation + recorded evidence. Later stages must extend `EngineSnapshot` and the versioned binary codec whenever they add new authoritative world, country, trade, military or war state.
