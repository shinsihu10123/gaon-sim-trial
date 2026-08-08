# Stage 3.1 — HumanGroup State Model

Status: IN DEVELOPMENT

Authoritative plan: v3.1 civilization-origin trial plan.

Stage 3 begins from HumanGroups that already exist in the Year-1 world. This stage does not create countries, settlements, or fixed civilization outcomes.

## Scope

- 3.1.1 HumanGroupState definition
- 3.1.2 population, position, and mobility fields
- 3.1.3 food, basic resources, and nutrition fields
- 3.1.4 cohesion and risk fields
- 3.1.5 Knowledge and Memory reference hooks
- 3.1.6 Parent / Split / Merge lineage structure

## Authority rules

HumanGroup runtime state belongs to Simulation Core. The Viewer may only render state exported through RenderSnapshot. No movement, settlement, survival outcome, split, merge, or country formation may be synthesized by the Viewer.

The first implementation slice defines and validates the full runtime-state schema without changing Save Format v6. A following Stage 3 slice will make this state authoritative and persistent together with an explicit save-format migration, avoiding silent loss of state through Save/Load.
