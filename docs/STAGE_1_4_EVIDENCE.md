# Stage 1.4 Evidence Record

## Scope

Stage 1.4 establishes the authoritative Event Ledger infrastructure required by later economic, policy, diplomatic, military and war systems.

The fixed architectural distinction is:

```text
Command = intent
Event = fact
```

Stage 1.4 does not fabricate domain events for systems that have not yet been implemented. Instead it fixes the immutable record format, category taxonomy, ordering, ownership, source attribution and filtering contract that those later systems will use.

## WBS coverage

- `1.4.1` Event Ledger structure — COMPLETE
- `1.4.2` Economic event category contract — COMPLETE
- `1.4.3` Policy event category contract — COMPLETE
- `1.4.4` Diplomatic event category contract — COMPLETE
- `1.4.5` Military event category contract — COMPLETE
- `1.4.6` War event category contract — COMPLETE
- `1.4.7` Occupation event category contract — COMPLETE
- `1.4.8` Treaty event category contract — COMPLETE
- `1.4.9` User intervention event recording — COMPLETE
- `1.4.10` Event filtering — COMPLETE

The category-contract items above mean that the ledger can represent, order and filter those event classes. Their domain-specific payload variants and emission rules remain intentionally assigned to the stages where the corresponding economic, diplomatic, military and war models are implemented.

## Implemented data contract

Each immutable `EventRecord` contains:

- monotonic `EventId`
- simulation occurrence date
- zero-based daily tick index
- `EventCategory`
- `EventSource`
- typed `EventPayload`

Current categories:

- `System`
- `Economic`
- `Policy`
- `Diplomatic`
- `Military`
- `War`
- `Occupation`
- `Treaty`
- `UserIntervention`

Current fact payload:

```text
CommandExecuted { command_id }
```

## Ownership and ordering

`SimulationEngine` is the sole owner of the Event Ledger.

The ledger is append-only. `EventId` values increase monotonically. Command-derived facts are appended immediately after deterministic command execution, so their order is inherited from the command key:

```text
(execution date, priority lane, command id)
```

For facts generated on the same daily tick, `EventId` is the final global ordering key.

## Source attribution

Current verified mapping:

- executed user command -> `UserIntervention` / `User`
- executed country-AI infrastructure command -> `System` / `Country(country_id)`

The second mapping deliberately avoids inventing a policy, economic or military meaning for the Stage 1 `NoOp` command.

## Filtering

`EventFilter` supports optional AND-combined filtering by:

- category
- source
- start date
- through date

Filtering is read-only and cannot mutate authoritative simulation state.

## GitHub validation

Authoritative validation was executed by GitHub Actions `Stage 1 Core CI` on pull request #2.

Passing run:

- workflow run id: `31158185323`
- validated feature head: `d4ac6985926268e9c65fa7e07f5744f94b9bdf4c`
- Rust: `1.97.1`

Passed gates:

1. `cargo fmt --all -- --check`
2. `cargo clippy --workspace --all-targets -- -D warnings`
3. `cargo test --workspace`
4. `cargo run -p simulation-runner --quiet`
5. Stage 1.3 source contract
6. Stage 1.4 Event Ledger source contract

Contract output:

```text
Stage 1.3 command contract verified
Stage 1.4 Event Ledger contract verified
```

## Test result

- simulation-core: **18 passed, 0 failed**
- simulation-model: **7 passed, 0 failed**
- simulation-save: **1 passed, 0 failed**
- simulation-runner unit tests: 0
- total Rust unit tests: **26 passed, 0 failed**

The core tests specifically verify:

- monotonic event IDs
- append-only record order
- user-intervention event creation
- country-AI source preservation
- event occurrence date and tick index
- category/source/date filtering
- identical command and event order under manual and high-speed playback

## Headless validation

```text
date=0002-01-01 elapsed_days=365 ticks=365 commands=2 events=2 render_stride=13 digest=0476ea3ebacedc22
```

The 365-day run executed both submitted commands and emitted exactly two corresponding immutable events while preserving the Stage 1.3 deterministic state digest.

## Decision

Status: **PASS**

Stage 1.4 satisfies the project completion rule of implementation + automated validation + recorded evidence. Domain-specific event payloads will be extended only when the corresponding later simulation systems exist.
