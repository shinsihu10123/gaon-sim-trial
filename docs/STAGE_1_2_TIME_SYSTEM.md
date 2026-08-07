# Stage 1.2 — Simulation Time and Fixed Daily Tick

## Purpose

Stage 1.2 establishes a deterministic simulation clock that is independent of wall-clock speed and renderer frame rate.

The authoritative rule is:

```text
1 simulation tick = 1 simulated day
```

Playback speed controls only how many daily ticks are processed per real second. It never changes the meaning or ordering of a tick.

## Calendar contract

The trial simulation starts at:

```text
Year 1, Month 1, Day 1
```

The Stage 1 calendar is a deterministic 365-day calendar with no leap years:

- January 31
- February 28
- March 31
- April 30
- May 31
- June 30
- July 31
- August 31
- September 30
- October 31
- November 30
- December 31

`SimulationDate::advance_one_day()` advances exactly one date and reports month, quarter and year boundaries.

The calendar is simulation data and does not read the host locale, timezone or system clock.

## Daily tick contract

A call to `SimulationEngine::tick()` advances exactly one simulated day.

At the current Stage 1 boundary the tick performs:

1. execute commands effective on the current date in deterministic order;
2. emit resulting Stage 1 events;
3. advance the simulation date by one day;
4. increment `elapsed_days` by one.

Later Stage 9 integration will insert the full economic, trade, military and other daily processing sequence around this fixed clock contract without changing the one-tick/one-day meaning.

## Playback speeds

Supported Stage 1 playback modes are:

- paused
- 1x = 1 simulated day / real second
- 7x = 7 simulated days / real second
- 30x = 30 simulated days / real second
- 90x = 90 simulated days / real second
- 365x = 365 simulated days / real second

`SimulationClock` converts elapsed real time into an integer number of daily ticks using integer nanosecond accumulation. Floating-point time accumulation is not used.

## Pause and manual stepping

Paused mode processes zero automatic ticks.

Manual controls are:

- `step_one_day()` = exactly 1 daily tick
- `step_one_month()` = exactly 30 daily ticks

The 30-day month step is a control shortcut defined by the TEST plan; it does not mean every calendar month has 30 days.

## Logic/render separation

Every simulation day is processed even at high speed. The renderer is not required to display every daily state.

`render_stride_days()` provides a presentation sampling interval:

- up to 30x: sample every simulated day
- 90x: sample every 3 simulated days
- 365x: sample every 13 simulated days

This keeps simulation logic exact while reducing visual snapshot frequency. Renderer sampling is not authoritative state and cannot alter simulation results.

## Determinism requirements

Stage 1.2 requires:

1. one tick advances exactly one simulated day;
2. calendar month/quarter/year boundaries are correct;
3. pause advances zero ticks;
4. each playback mode maps to the planned number of daily ticks;
5. fractional real-time intervals accumulate without floating-point drift;
6. manual 30-day step advances exactly 30 ticks;
7. high-speed rendering may skip presentation samples but never simulation logic;
8. identical tick counts produce identical authoritative state regardless of playback speed.

## Stage boundary

Stage 1.2 defines time progression only. Economic monthly updates, quarterly GDP calculations, yearly demographic/education processing and event-driven war/diplomatic processing are added by their respective domain stages while consuming these fixed calendar boundary signals.
