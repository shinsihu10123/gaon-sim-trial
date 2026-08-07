# Stage 1.3 — Deterministic Command Processing

## Purpose

Stage 1.3 establishes the command infrastructure that every later policy, diplomacy, military and AI action will use. It intentionally does **not** invent those later domain commands before their models exist.

## Canonical command envelope

Every accepted command contains:

- monotonic `CommandId`
- `CommandSource`
  - `User`
  - `CountryAi(CountryId)`
- submission date
- resolved execution date
- fixed priority lane
- payload

The Stage 1 payload currently contains only `NoOp { token }`, an infrastructure health probe. It exists so the queue, ordering, logging and future replay path can be tested without introducing fake economic or military state.

## Timing semantics

Two request modes exist:

- `Immediate`: effective on the **current simulation date** and processed at the start of the next deterministic daily tick.
- `Scheduled(date)`: processed when that simulation date becomes current.

An immediate command is therefore not a mid-tick mutation. If the simulation is paused, the accepted command remains pending until the user advances or resumes time. This keeps the simulation core as the sole state mutator.

Scheduled dates must:

1. be valid under the fixed 365-day simulation calendar;
2. not be earlier than the current simulation date.

Invalid requests are rejected before queue mutation and return a typed `CommandError`.

## Deterministic ordering

The pending queue is sorted by the canonical key:

```text
(execution date, priority lane, command id)
```

Current Stage 1 lanes are:

| Priority | Source | Meaning |
| ---: | --- | --- |
| 10 | User | Explicit user intervention |
| 20 | Country AI | Autonomous country decision |

Lower values execute first. Therefore a user intervention takes precedence over autonomous AI commands on the same simulation date. Commands in the same lane preserve submission order through monotonic `CommandId` values.

This ordering is independent of renderer frame rate and playback speed.

## Execution record

Every executed command appends a `CommandExecutionRecord` containing:

- the complete accepted command envelope;
- the actual execution date.

This is the Stage 1 command history. The separate Event Ledger in Stage 1.4 will record world facts and effects; commands and events remain distinct concepts.

## API surface

The engine exposes:

- `submit_command`
- `submit_user_command`
- `submit_country_ai_command`
- `pending_commands`
- `command_log`

Only the engine assigns IDs, resolves dates, determines priority and executes queued commands.

## Validation requirements

Rust tests cover:

- immediate user command execution;
- future scheduled execution without early application;
- user-before-AI same-day priority;
- same-priority submission-order stability;
- invalid calendar-date rejection;
- past-date rejection without queue mutation;
- source recording for country AI commands;
- identical command order under manual ticks and high-speed playback.

Offline contract checks verify the required source-level invariants when a Rust toolchain is unavailable locally. GitHub CI remains the authoritative runtime compiler/test gate.
