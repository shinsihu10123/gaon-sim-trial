import * as THREE from "three";
import type { RenderHumanGroupSnapshot, RenderRegionSnapshot, RenderWorldSnapshot } from "./types";
import { WorldSpaceTransform } from "./world-space";

export const SELECTABLE_ENTITY_KINDS = [
  "human_group",
  "settlement",
  "community",
  "political_entity",
  "country",
  "region",
] as const;

export type SelectableEntityKind = (typeof SELECTABLE_ENTITY_KINDS)[number];

export interface EntitySelectionTarget {
  kind: SelectableEntityKind;
  id: string;
  scenePoint: THREE.Vector3;
}

export class EntitySelectionLayer {
  private readonly pickGroup = new THREE.Group();
  private readonly highlightGroup = new THREE.Group();
  private readonly raycaster = new THREE.Raycaster();
  private readonly pointer = new THREE.Vector2();
  private readonly targets = new Map<string, EntitySelectionTarget>();
  private readonly regions = new Map<string, RenderRegionSnapshot>();
  private transform: WorldSpaceTransform | undefined;
  private selectedKey: string | undefined;

  public constructor(private readonly scene: THREE.Scene) {
    this.pickGroup.name = "entity-selection-hit-proxies";
    this.highlightGroup.name = "entity-selection-highlight";
    scene.add(this.pickGroup, this.highlightGroup);
  }

  public update(world: RenderWorldSnapshot): void {
    this.clearPickTargets();
    this.clearHighlight();
    this.targets.clear();
    this.regions.clear();
    this.selectedKey = undefined;
    this.transform = world.bounds === null ? undefined : new WorldSpaceTransform(world.bounds);
    if (this.transform === undefined) {
      return;
    }

    for (const region of world.regions) {
      this.registerRegion(region);
    }
    for (const group of world.humanGroups) {
      this.registerHumanGroup(group, world);
    }
  }

  public listTargets(): EntitySelectionTarget[] {
    return [...this.targets.values()].map((target) => ({
      ...target,
      scenePoint: target.scenePoint.clone(),
    }));
  }

  public findTarget(kind: SelectableEntityKind, id: string): EntitySelectionTarget | undefined {
    const target = this.targets.get(selectionKey(kind, id));
    if (target === undefined) {
      return undefined;
    }
    return { ...target, scenePoint: target.scenePoint.clone() };
  }

  public pickFromClientPoint(
    clientX: number,
    clientY: number,
    camera: THREE.Camera,
    element: HTMLElement,
  ): EntitySelectionTarget | undefined {
    const rect = element.getBoundingClientRect();
    if (rect.width <= 0 || rect.height <= 0) {
      return undefined;
    }
    this.pointer.x = ((clientX - rect.left) / rect.width) * 2 - 1;
    this.pointer.y = -((clientY - rect.top) / rect.height) * 2 + 1;
    this.raycaster.setFromCamera(this.pointer, camera);

    const intersections = this.raycaster.intersectObjects(this.pickGroup.children, true);
    for (const intersection of intersections) {
      const key = intersection.object.userData.selectionKey;
      if (typeof key !== "string") {
        continue;
      }
      const target = this.targets.get(key);
      if (target !== undefined) {
        return { ...target, scenePoint: target.scenePoint.clone() };
      }
    }
    return undefined;
  }

  public select(target: EntitySelectionTarget | undefined): void {
    this.clearHighlight();
    this.selectedKey = target === undefined ? undefined : selectionKey(target.kind, target.id);
    if (target === undefined || this.transform === undefined) {
      return;
    }

    if (target.kind === "region") {
      const region = this.regions.get(target.id);
      if (region !== undefined) {
        this.highlightGroup.add(this.createRegionHighlight(region));
        return;
      }
    }
    this.highlightGroup.add(this.createPointReticle(target.scenePoint));
  }

  public selectedTarget(): EntitySelectionTarget | undefined {
    if (this.selectedKey === undefined) {
      return undefined;
    }
    const target = this.targets.get(this.selectedKey);
    return target === undefined ? undefined : { ...target, scenePoint: target.scenePoint.clone() };
  }

  public dispose(): void {
    this.clearPickTargets();
    this.clearHighlight();
    this.targets.clear();
    this.regions.clear();
    this.pickGroup.removeFromParent();
    this.highlightGroup.removeFromParent();
  }

  private registerRegion(region: RenderRegionSnapshot): void {
    if (this.transform === undefined || region.boundary.length < 3) {
      return;
    }
    const focus = this.transform.worldPointToScene(region.center.xM, region.center.zM, 0.18);
    const target: EntitySelectionTarget = { kind: "region", id: region.id, scenePoint: focus };
    const key = selectionKey(target.kind, target.id);
    this.targets.set(key, target);
    this.regions.set(region.id, region);

    const shape = new THREE.Shape();
    const first = this.transform.worldPointToScene(region.boundary[0].xM, region.boundary[0].zM);
    shape.moveTo(first.x, -first.z);
    for (const point of region.boundary.slice(1)) {
      const scenePoint = this.transform.worldPointToScene(point.xM, point.zM);
      shape.lineTo(scenePoint.x, -scenePoint.z);
    }
    shape.closePath();

    const geometry = new THREE.ShapeGeometry(shape);
    const material = new THREE.MeshBasicMaterial({
      side: THREE.DoubleSide,
      transparent: true,
      opacity: 0,
      depthWrite: false,
    });
    material.colorWrite = false;
    const mesh = new THREE.Mesh(geometry, material);
    mesh.rotation.x = -Math.PI / 2;
    mesh.position.y = 0.12;
    mesh.name = `region-hit-${region.id}`;
    mesh.userData.selectionKey = key;
    this.pickGroup.add(mesh);
  }

  private registerHumanGroup(group: RenderHumanGroupSnapshot, world: RenderWorldSnapshot): void {
    if (this.transform === undefined) {
      return;
    }
    const sceneY = sceneHeightAt(world, this.transform, group.xM, group.zM) + 0.35;
    const point = this.transform.worldPointToScene(group.xM, group.zM, sceneY);
    const target: EntitySelectionTarget = {
      kind: "human_group",
      id: group.id,
      scenePoint: point,
    };
    const key = selectionKey(target.kind, target.id);
    this.targets.set(key, target);

    const geometry = new THREE.SphereGeometry(0.75, 10, 8);
    const material = new THREE.MeshBasicMaterial({ transparent: true, opacity: 0, depthWrite: false });
    material.colorWrite = false;
    const proxy = new THREE.Mesh(geometry, material);
    proxy.position.copy(point);
    proxy.name = `human-group-hit-${group.id}`;
    proxy.userData.selectionKey = key;
    this.pickGroup.add(proxy);
  }

  private createRegionHighlight(region: RenderRegionSnapshot): THREE.LineLoop {
    if (this.transform === undefined) {
      throw new Error("selection transform unavailable");
    }
    const points = region.boundary.map((point) => {
      const scenePoint = this.transform?.worldPointToScene(point.xM, point.zM, 0.22);
      if (scenePoint === undefined) {
        throw new Error("selection transform unavailable");
      }
      return scenePoint;
    });
    const geometry = new THREE.BufferGeometry().setFromPoints(points);
    const material = new THREE.LineBasicMaterial({ color: 0xffffff, depthTest: false });
    const line = new THREE.LineLoop(geometry, material);
    line.name = `selected-region-${region.id}`;
    line.renderOrder = 20;
    return line;
  }

  private createPointReticle(point: THREE.Vector3): THREE.Mesh {
    const geometry = new THREE.RingGeometry(0.8, 1.05, 28);
    const material = new THREE.MeshBasicMaterial({
      color: 0xffffff,
      side: THREE.DoubleSide,
      depthTest: false,
    });
    const ring = new THREE.Mesh(geometry, material);
    ring.rotation.x = -Math.PI / 2;
    ring.position.copy(point);
    ring.name = "selected-entity-reticle";
    ring.renderOrder = 20;
    return ring;
  }

  private clearPickTargets(): void {
    disposeChildren(this.pickGroup);
  }

  private clearHighlight(): void {
    disposeChildren(this.highlightGroup);
  }
}

export function selectionKey(kind: SelectableEntityKind, id: string): string {
  return `${kind}:${id}`;
}

function sceneHeightAt(
  world: RenderWorldSnapshot,
  transform: WorldSpaceTransform,
  xM: number,
  zM: number,
): number {
  const terrain = world.terrain;
  if (terrain === null || terrain.spacingM <= 0) {
    return 0;
  }
  const x = Math.round((xM - transform.bounds.minXM) / terrain.spacingM);
  const z = Math.round((zM - transform.bounds.minZM) / terrain.spacingM);
  if (x < 0 || z < 0 || x >= terrain.width || z >= terrain.height) {
    return 0;
  }
  const elevation = terrain.elevationM[z * terrain.width + x];
  return transform.elevationToSceneY(elevation ?? 0);
}

function disposeChildren(group: THREE.Group): void {
  for (const child of [...group.children]) {
    child.removeFromParent();
    if (child instanceof THREE.Mesh || child instanceof THREE.Line) {
      child.geometry.dispose();
      disposeMaterial(child.material);
    }
  }
}

function disposeMaterial(material: THREE.Material | THREE.Material[]): void {
  if (Array.isArray(material)) {
    for (const item of material) {
      item.dispose();
    }
  } else {
    material.dispose();
  }
}
