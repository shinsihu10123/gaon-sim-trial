import type { EntitySelectionTarget, SelectableEntityKind } from "./entity-selection";
import { selectionKey } from "./entity-selection";

export class EntitySelectionPanel {
  public readonly element = document.createElement("aside");
  private readonly select = document.createElement("select");
  private readonly focusButton = document.createElement("button");
  private readonly worldButton = document.createElement("button");
  private readonly summary = document.createElement("div");
  private readonly targets = new Map<string, EntitySelectionTarget>();

  public onSelectionChanged: ((target: EntitySelectionTarget | undefined) => void) | undefined;
  public onFocusRequested: ((target: EntitySelectionTarget) => void) | undefined;
  public onWorldRequested: (() => void) | undefined;

  public constructor(root: HTMLElement) {
    this.element.className = "selection-panel";

    const title = document.createElement("div");
    title.className = "selection-panel__title";
    title.textContent = "Entity Selection";

    const controls = document.createElement("div");
    controls.className = "selection-panel__controls";

    this.select.className = "selection-panel__select";
    this.select.setAttribute("aria-label", "Select simulation entity");
    this.select.addEventListener("change", () => {
      const target = this.targets.get(this.select.value);
      this.setSummary(target);
      this.onSelectionChanged?.(target);
    });

    this.focusButton.type = "button";
    this.focusButton.textContent = "Focus";
    this.focusButton.disabled = true;
    this.focusButton.addEventListener("click", () => {
      const target = this.targets.get(this.select.value);
      if (target !== undefined) {
        this.onFocusRequested?.(target);
      }
    });

    this.worldButton.type = "button";
    this.worldButton.textContent = "World";
    this.worldButton.addEventListener("click", () => this.onWorldRequested?.());

    this.summary.className = "selection-panel__summary";
    this.setSummary(undefined);

    controls.append(this.select, this.focusButton, this.worldButton);
    this.element.append(title, controls, this.summary);
    root.append(this.element);
  }

  public setTargets(targets: EntitySelectionTarget[]): void {
    const previous = this.select.value;
    this.targets.clear();
    this.select.replaceChildren();

    const empty = document.createElement("option");
    empty.value = "";
    empty.textContent = "Select entity…";
    this.select.append(empty);

    const ordered = [...targets].sort((left, right) => {
      const kindOrder = kindRank(left.kind) - kindRank(right.kind);
      if (kindOrder !== 0) return kindOrder;
      return left.id.localeCompare(right.id, undefined, { numeric: true });
    });

    for (const target of ordered) {
      const key = selectionKey(target.kind, target.id);
      this.targets.set(key, target);
      const option = document.createElement("option");
      option.value = key;
      option.textContent = `${kindLabel(target.kind)} · ${target.id}`;
      this.select.append(option);
    }

    if (previous !== "" && this.targets.has(previous)) {
      this.select.value = previous;
      const target = this.targets.get(previous);
      this.setSummary(target);
    } else {
      this.select.value = "";
      this.setSummary(undefined);
    }
  }

  public setSelected(target: EntitySelectionTarget | undefined): void {
    const key = target === undefined ? "" : selectionKey(target.kind, target.id);
    this.select.value = this.targets.has(key) ? key : "";
    this.setSummary(target);
  }

  public dispose(): void {
    this.element.remove();
    this.targets.clear();
  }

  private setSummary(target: EntitySelectionTarget | undefined): void {
    this.focusButton.disabled = target === undefined;
    this.summary.textContent = target === undefined
      ? "No entity selected"
      : `${kindLabel(target.kind)} ${target.id}`;
  }
}

function kindRank(kind: SelectableEntityKind): number {
  switch (kind) {
    case "human_group": return 0;
    case "settlement": return 1;
    case "community": return 2;
    case "political_entity": return 3;
    case "country": return 4;
    case "region": return 5;
  }
}

function kindLabel(kind: SelectableEntityKind): string {
  switch (kind) {
    case "human_group": return "HumanGroup";
    case "settlement": return "Settlement";
    case "community": return "Community";
    case "political_entity": return "PoliticalEntity";
    case "country": return "Country";
    case "region": return "Region";
  }
}
