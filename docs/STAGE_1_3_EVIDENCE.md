# Stage 1.3 Evidence Record

## WBS scope

- `1.3.1` Command common interface
- `1.3.2` User command validation
- `1.3.3` Country-AI command structure
- `1.3.4` Immediate and scheduled execution
- `1.3.5` Command application priority
- `1.3.6` Executed-command recording
- `1.3.7` Invalid-command rejection and typed error return

## Implemented

- Common `CommandRequest` and accepted `QueuedCommand` structures
- User and autonomous country-AI command origins
- Immediate and scheduled execution dates
- Deterministic same-day priority lanes
- Monotonic `CommandId` tie-breaking
- Typed rejection of invalid and past scheduled dates
- Append-only `CommandExecutionRecord` history
- Daily-tick command processing before calendar advancement
- Manual-vs-high-speed command-order determinism test

## Command-order invariant

Accepted commands are executed by the canonical ordering key:

```text
(execution date, priority lane, command id)
```

On the same simulation date, explicit user intervention precedes autonomous country-AI commands. Within the same priority lane, monotonically assigned command IDs preserve submission order.

## Local validation

The current execution environment does not provide `cargo` or `rustc`, so Rust runtime validation could not be executed locally.

The source-level Stage 1.3 contract check passed locally:

```text
Stage 1.3 command contract verified
```

## GitHub validation

Authoritative validation was executed by GitHub Actions `Stage 1 Core CI` on pull request #1.

Passing run:

- workflow run id: `31157199949`
- validated develop head: `197e2bf31af9688d2e6ca427820bec7970b66581`
- Rust: `1.97.1`

Passed gates:

1. `cargo fmt --all -- --check`
2. `cargo clippy --workspace --all-targets -- -D warnings`
3. `cargo test --workspace`
4. `cargo run -p simulation-runner --quiet`
5. `node scripts/check-command-contract.mjs`

Test result:

- simulation-core: 14 passed, 0 failed
- simulation-model: 5 passed, 0 failed
- simulation-save: 1 passed, 0 failed
- total Rust unit tests: **20 passed, 0 failed**

Headless trial result:

```text
date=0002-01-01 elapsed_days=365 ticks=365 commands=2 render_stride=13 digest=0476ea3ebacedc22
```

This confirms that the test runner processed 365 deterministic daily ticks, executed both submitted commands, maintained the high-speed render stride, and produced a stable state digest.

## WBS decision

Status: **PASS**

- `1.3.1` COMPLETE
- `1.3.2` COMPLETE
- `1.3.3` COMPLETE
- `1.3.4` COMPLETE
- `1.3.5` COMPLETE
- `1.3.6` COMPLETE
- `1.3.7` COMPLETE

Stage 1.3 is complete under the project rule of implementation + validation + recorded evidence.
