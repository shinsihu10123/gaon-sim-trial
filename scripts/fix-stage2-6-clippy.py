from pathlib import Path

entity_path = Path("crates/simulation-model/src/entity.rs")
entity_text = entity_path.read_text()
entity_text = entity_text.replace("Restores a HumanGroup registry", "Restores a `HumanGroup` registry")
entity_text = entity_text.replace("Creates an uninitialized HumanGroup identity", "Creates an uninitialized `HumanGroup` identity")
entity_text = entity_text.replace("Creates a Year-1 HumanGroup with", "Creates a Year-1 `HumanGroup` with")
entity_text = entity_text.replace("Removes an existing HumanGroup without", "Removes an existing `HumanGroup` without")
entity_path.write_text(entity_text)

save_path = Path("crates/simulation-save/src/lib.rs")
save_text = save_path.read_text()
save_text = save_text.replace(
    "Stage 2.6 adds authoritative Year-1 HumanGroup ecological seed state.",
    "Stage 2.6 adds authoritative Year-1 `HumanGroup` ecological seed state.",
)
save_path.write_text(save_text)
