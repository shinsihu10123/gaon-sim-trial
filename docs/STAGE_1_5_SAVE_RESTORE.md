# Stage 1.5 — Save / Restore

## Purpose

Stage 1.5 makes a running simulation recoverable without changing its future deterministic trajectory.

A valid restore must recover more than the visible date. It must restore every authoritative Stage 1 component that can affect later ticks:

- `WorldState`
- pending command queue
- executed command history
- Event Ledger
- next `CommandId`
- next `EventId`

Playback speed and renderer timing are intentionally excluded because they are presentation controls, not authoritative simulation state.

## File contract

The logical save consists of two parts:

```text
metadata.json
state.bin
```

The API exposes them as `SaveBundle.metadata_json` and `SaveBundle.state_binary`; a browser or desktop storage adapter may persist or package them without changing their contents.

### metadata.json

Human-readable metadata contains:

- save-format version
- save kind (`manual` or `autosave`)
- simulation date
- elapsed simulated days
- seed encoded as fixed-width hexadecimal text
- binary payload length
- FNV-1a 64-bit binary integrity checksum

The seed is text rather than a JSON integer so a future TypeScript UI cannot lose 64-bit precision.

The checksum is an accidental-corruption detector, not a cryptographic signature.

### state.bin

`state.bin` is a canonical little-endian binary representation.

It begins with:

```text
magic = GAONST01
format version = 1
```

It then serializes the authoritative engine state in fixed field order. Enum values use explicit stable tags rather than compiler-dependent memory layouts.

The decoder rejects:

- invalid magic
- unsupported binary version
- truncated payloads
- trailing bytes
- unknown enum tags
- unsafe record counts
- invalid simulation dates

## Snapshot invariants

Before saving and after decoding, the snapshot is validated.

Current invariants include:

1. date and elapsed-day counter agree exactly;
2. command IDs are unique and nonzero;
3. the pending queue is in canonical `(execution date, priority, command id)` order;
4. pending commands are not already overdue;
5. command priority agrees with command source;
6. executed-command dates agree with their resolved execution dates;
7. next command ID follows accepted command history;
8. Event IDs preserve contiguous append order;
9. Event dates and tick positions are within completed simulation time;
10. every `CommandExecuted` fact references a real executed command;
11. command-execution event attribution agrees with the command source;
12. next Event ID follows the Event Ledger.

These checks prevent a syntactically decodable but internally inconsistent save from becoming authoritative state.

## Manual save and autosave

Manual and autosave snapshots use the same format and validation pipeline. `SaveKind` records the origin only; an autosave is not a lower-quality snapshot.

`AutosavePolicy` provides a simulated-day cadence helper. The caller records the day of the last *successful* autosave, so a failed persistence attempt cannot silently advance the schedule.

The policy does not mutate simulation state and therefore cannot change deterministic results.

## Restore semantics

Restore is transactional from the engine's perspective:

1. parse metadata;
2. validate metadata version;
3. validate binary length;
4. verify checksum;
5. decode canonical binary state;
6. validate snapshot invariants;
7. verify metadata and binary state agree;
8. construct a new `SimulationEngine` only after all checks pass.

A malformed or corrupted bundle therefore cannot partially mutate an existing engine.

## Long-run validation

Stage 1.5 requires a 100-year continuation test:

1. create a deterministic engine;
2. execute an immediate command;
3. schedule another command for Year 75;
4. run 50 simulated years;
5. create an autosave;
6. restore a second engine from that autosave;
7. assert exact snapshot equality at the restore point;
8. advance both engines for another 50 years;
9. assert exact snapshot, command-log, Event-Ledger and state-digest equality at Year 101.

This test deliberately includes a command that is still pending at the save point and executes after restoration. It therefore detects incomplete command-queue persistence that a simple date-only save test would miss.

## Stage boundary

Stage 1.5 establishes exact persistence for the authoritative structures that exist today. When later stages add regions, countries, trade, armies and wars, their authoritative fields must be added to `EngineSnapshot` and the versioned codec before those stages can satisfy persistence acceptance criteria.
