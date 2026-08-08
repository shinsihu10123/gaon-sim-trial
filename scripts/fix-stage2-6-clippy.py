from pathlib import Path

path = Path("crates/simulation-model/src/entity.rs")
text = path.read_text()
text = text.replace("Restores a HumanGroup registry", "Restores a `HumanGroup` registry")
text = text.replace("Creates an uninitialized HumanGroup identity", "Creates an uninitialized `HumanGroup` identity")
text = text.replace("Creates a Year-1 HumanGroup with", "Creates a Year-1 `HumanGroup` with")
text = text.replace("Removes an existing HumanGroup without", "Removes an existing `HumanGroup` without")
path.write_text(text)
