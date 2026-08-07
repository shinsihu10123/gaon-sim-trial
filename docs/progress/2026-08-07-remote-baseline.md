# 2026-08-07 Remote Baseline Evidence

## Scope

WBS item: `1.1.1 GitHub 저장소 생성 및 브랜치 전략 설정`

## Repository

- Repository: `shinsihu10123/gaon-sim-trial`
- Visibility: Private
- Default branch: `main`
- Integration branch: `develop`

## Branch strategy

- `main`: validated baseline
- `develop`: integration development
- `feature/<scope>`: feature implementation
- `fix/<scope>`: defect correction

Detailed rules are recorded in `docs/BRANCHING.md`.

## Recovery baseline

The complete local Git state through trial v1.2 is preserved at:

`backups/gaon-sim-trial-v1.2.bundle.b64`

Original Git bundle SHA-256:

`2eb951e31f4989bb9b0984ebc64e16da165ba0f340ea2454f8cf29e9b239ffcd`

Recovery instructions are recorded in `RESTORE.md`.

## Source import verification

The v1.2 source tree has been restored into the GitHub repository and verified remotely. Representative paths include:

- `Cargo.toml`
- `crates/simulation-core/src/lib.rs`
- `viewer/package.json`

The same source baseline is available on both `main` and `develop`.

## WBS decision

`1.1.1 GitHub 저장소 생성 및 브랜치 전략 설정` -> **COMPLETE**

Completion evidence satisfies the project rule:

1. Repository and branch artifacts exist.
2. Remote source and recovery paths were read back from GitHub.
3. This evidence record and GitHub history provide auditable proof.

Other 1.1 items remain incomplete until their own build/runtime/CI acceptance conditions are verified.
