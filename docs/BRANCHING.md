# Branching Strategy

## Permanent branches

- `main`: validated baseline. Only states that have passed the relevant acceptance checks should remain here.
- `develop`: integration branch for ongoing trial development.

## Working branches

- `feature/<scope>`: implementation of one WBS feature or tightly related set of features.
- `fix/<scope>`: defect correction.

## Promotion rule

1. Start feature work from `develop`.
2. Implement the relevant WBS items.
3. Run the available unit, integration, determinism, and structural checks.
4. Merge the verified feature into `develop`.
5. Promote `develop` to `main` only at a meaningful validated checkpoint.
6. Record the corresponding WBS status and evidence.

A feature is not considered complete merely because source code exists. The project completion rule remains: implementation + validation + evidence.
