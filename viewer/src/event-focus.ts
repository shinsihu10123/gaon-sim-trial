import type { SelectableEntityKind } from "./entity-selection";
import type { WorldCameraController } from "./world-camera";

export const AUTO_FOCUS_EVENT_KINDS = ["war", "mass_migration"] as const;
export type AutoFocusEventKind = (typeof AUTO_FOCUS_EVENT_KINDS)[number];

export interface SpatialFocusEvent {
  kind: AutoFocusEventKind;
  xM: number;
  zM: number;
  entity?: {
    kind: SelectableEntityKind;
    id: string;
  };
}

export class SpatialEventFocusController {
  public enabled = true;
  private lastFocused: SpatialFocusEvent | undefined;

  public constructor(private readonly camera: WorldCameraController) {}

  public handle(event: SpatialFocusEvent): boolean {
    if (!this.enabled || !AUTO_FOCUS_EVENT_KINDS.includes(event.kind)) {
      return false;
    }
    this.lastFocused = {
      ...event,
      entity: event.entity === undefined ? undefined : { ...event.entity },
    };
    this.camera.focusWorldPoint(event.xM, event.zM);
    return true;
  }

  public lastFocusedEvent(): SpatialFocusEvent | undefined {
    return this.lastFocused === undefined
      ? undefined
      : {
          ...this.lastFocused,
          entity: this.lastFocused.entity === undefined ? undefined : { ...this.lastFocused.entity },
        };
  }
}
