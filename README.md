# Gaon Multi-Nation Simulation — Trial Build

경제 동아리 「가온」의 AI 기반 다국가 통합 발전 시뮬레이션 시험판 저장소입니다.

## Trial scope

- One continuous 3D world
- 10 autonomous countries and 60 regions
- One simulation tick per day
- Economy, population, government, education, technology, diplomacy, military, war
- User intervention and counterfactual baseline branching
- Deterministic execution and state hashing
- Three.js-based 3D observation layer

## Repository layout

```text
crates/
  simulation-model/   Shared deterministic domain model
  simulation-core/    Fixed-tick simulation engine
  simulation-save/    Save metadata and text snapshot utilities
  simulation-runner/  Headless command-line runner
viewer/                TypeScript, Vite and Three.js client
shared/contracts/      Engine-renderer data contracts
docs/                  Architecture, workflow and scope documents
scripts/               Repository validation scripts
.github/workflows/     CI configuration
```

## Local prerequisites

- Rust stable toolchain
- Node.js 22 or newer
- npm 10 or newer
- Git

## First run

```bash
cargo test --workspace
npm --prefix viewer install
npm --prefix viewer run check
npm --prefix viewer run test
npm --prefix viewer run dev
```

## Validation

```bash
node scripts/check-structure.mjs
node scripts/check-all.mjs
```

The current repository contains the Stage 1 foundation and Stage 1.2 deterministic time-system implementation. It does not yet implement the full simulation described in the trial plan.
