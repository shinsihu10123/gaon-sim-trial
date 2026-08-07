# Stage 1.6 — Determinism, State Digest and Replay

## Purpose

Stage 1.6 makes reproducibility an explicit simulation invariant rather than an incidental property of the current minimal model.

For the same model version, seed and accepted command journal, the simulation must produce the same authoritative state regardless of renderer sampling frequency or playback speed.

## Authoritative state digest

The canonical **authoritative state digest** is computed from the same validated binary representation used by Stage 1.5 save/restore.

It therefore includes every authoritative Stage 1 field:

- world seed, date and elapsed days
- pending commands
- executed-command history
- Event Ledger
- next `CommandId`
- next `EventId`

This closes a weakness of the early development digest, which only covered the minimal world clock/seed state. The early digest may remain temporarily for compatibility with earlier tests, but it is no longer an acceptance or reproducibility criterion after Stage 1.6.

A pending command must change the authoritative digest even before that command executes.

## Deterministic random contract

Stage 1.6 adopts **counter-based random** generation for future stochastic domain logic.

A sample is a pure function of:

```text
world seed
+ current simulation tick
+ stable stream identifier
+ caller ordinal
```

There is no mutable global RNG cursor.

This prevents an unrelated extra random call in one subsystem from shifting the random values later consumed by another subsystem. For example, adding a visual or diplomatic random draw must not silently change an economic or combat random outcome when those systems use distinct stable streams.

Future systems must assign stable stream identifiers and stable ordinals from domain semantics rather than from renderer call order.

## Command journal replay

`accepted_command_journal()` merges executed and still-pending commands and sorts them by monotonic `CommandId`, preserving original acceptance order.

**command journal replay** starts from a fresh engine with the original seed, re-submits accepted commands on their recorded submission dates, preserves their effective dates and IDs, and advances to the requested target tick.

A valid replay must reconstruct:

- current world state
- pending command queue
- executed-command history
- Event Ledger
- next IDs
- authoritative state digest

The replay path uses recorded accepted commands, not UI state or renderer state.

## Replay edge semantics

Commands recorded as originally immediate are replayed as scheduled commands with the same resolved execution date. This is intentional: deterministic replay reproduces the accepted command fact (`submitted_on`, `execute_on`, source, payload and ID), not the UI gesture that originally produced it.

Commands accepted on the target state's current date but not yet followed by a daily tick are also re-submitted before replay terminates. This allows an exact replay of snapshots containing newly accepted pending commands.

Malformed journals fail replay rather than being silently normalized. Examples include non-contiguous command IDs, impossible submission dates or commands that cannot be re-accepted under the recorded execution date.

## Required validation

Stage 1.6 must verify all of the following:

1. repeated random requests with identical keys return identical values;
2. random output is independent of request call order;
3. changing seed, tick, stream or ordinal changes the tested sample;
4. a pending command changes the authoritative digest;
5. manual and 365x playback produce the same authoritative digest;
6. save/restore preserves the authoritative digest;
7. a command journal reconstructs the exact authoritative snapshot;
8. a **10,000-tick replay** reconstructs the exact snapshot and digest;
9. repeated same-seed/same-journal runs remain identical;
10. renderer sampling never becomes part of authoritative state.

## State ownership

The simulation core remains the sole owner of authoritative state and deterministic random generation. The UI may request commands and read snapshots but must not own an RNG that can influence simulation results.

## Stage boundary

Stage 1.6 establishes deterministic infrastructure, not stochastic economic or military behavior. Later systems may consume counter-based random samples, but each domain must define its stable stream and ordinal semantics as part of that domain's own design and acceptance tests.
