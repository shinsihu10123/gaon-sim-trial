# Stage 1.4 — Event Ledger

## Purpose

The Event Ledger is the authoritative chronological record of facts that have actually occurred inside the simulation.

The architectural distinction is fixed:

```text
Command = intent
Event = fact
```

A command requests an action. An event records a fact produced by simulation processing. Commands and events therefore remain separate data structures and separate histories.

## Event record contract

Every `EventRecord` contains:

- monotonic `EventId`
- simulation date of occurrence
- zero-based daily tick index
- stable event category
- event source
- typed payload

Records are immutable after append. The ledger is append-only and preserves canonical event order.

## Event categories

The Stage 1 taxonomy reserves the categories required by later simulation systems:

- `System`
- `Economic`
- `Policy`
- `Diplomatic`
- `Military`
- `War`
- `Occupation`
- `Treaty`
- `UserIntervention`

These categories define the storage and filtering contract only. Stage 1 does not fabricate economic, diplomatic, military, war, occupation or treaty events before those domain systems exist.

## Stage 1 payload

The only fact currently emitted is:

```text
CommandExecuted { command_id }
```

This event establishes a verified connection between the deterministic command queue and the immutable world history.

Current mapping:

- executed user command → category `UserIntervention`, source `User`
- executed country-AI infrastructure command → category `System`, source `Country(country_id)`

A future domain command may emit one or more additional typed domain events after the relevant model exists.

## Ordering

Events are appended during deterministic daily processing. Because due commands execute in canonical command order and each completed command immediately emits its execution fact, command-derived events inherit deterministic ordering.

`EventId` is the global final ordering key for facts recorded at the same tick.

## Filtering

`EventFilter` supports optional AND-combined filters for:

- event category
- event source
- start date
- through date

Filtering is read-only and cannot mutate authoritative state.

## Ownership

`SimulationEngine` owns the Event Ledger. UI and render layers may read or filter events but cannot append arbitrary facts. Later core domain modules will emit events through the simulation core rather than through presentation code.

## Validation requirements

Stage 1.4 validation must prove:

1. event IDs increase monotonically;
2. records are append-only;
3. executed user commands emit `UserIntervention` events;
4. country-AI infrastructure commands retain their country source;
5. event dates and tick indices correspond to actual execution time;
6. category/source/date filtering returns the expected records;
7. manual and high-speed playback produce identical event order for the same seed and command sequence;
8. all required future event categories exist without fabricating unsupported domain facts.
