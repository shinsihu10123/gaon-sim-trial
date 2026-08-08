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
  private readonly proxies = new Map<string, THREE.Object3D>();
  private transform: WorldSpaceTransform | undefined;
  private selectedKey: string | undefined;

  public constructor(private readonly scene: THREE.Scene) {
    this.pickGroup.name = "entity-selection-hit-proxies";
    this.highlightGroup.name = "entity-selection-highlight";
    scene.add(this.pickGroup, this.highlightGroup);
  }

  public update(world: RenderWorldSnapshot): void {
    const previousSelectedKey = this.selectedKey;
    this.targets.clear();
    this.regions.clear();
    this.transform = world.bounds === null ? undefined : new WorldSpaceTransform(world.bounds);
    if (this.transform === undefined) {
      this.clearPickTargets();
      this.clearHighlight();
      this.selectedKey = undefined;
      return;
    }

    const activeKeys = new Set<string>();
    for (const region of world.regions) {
      const key = this.registerRegion(region);
      if (key !== undefined) activeKeys.add(key);
    }
    for (const group of world.humanGroups) {
      const key = this.registerHumanGroup(group, world);
      if (key !== undefined) activeKeys.add(key);
    }
    for (const [key, proxy] of [...this.proxies.entries()]) {
      if (!activeKeys.has(key)) {
        disposeObject(proxy);
        this.proxies.delete(key);
      }
    }

    this.clearHighlight();
    if (previousSelectedKey !== undefined) {
      const target = this.targets.get(previousSelectedKey);
      if (target !== undefined) {
        this.selectedKey = previousSelectedKey;
        this.renderHighlight(target);
        return;
      }
    }
    this.selectedKey = undefined;
  }

  public listTargets(): EntitySelectionTarget[] {
    return [...this.targets.values()].map((target) => ({
      ...target,
      scenePoint: target.scenePoint.clone(),
    }));
  }

  public findTarget(kind: SelectableEntityKind, id: string): EntitySelectionTarget | undefined {
    const target = this.targets.get(selectionKey(kind, id));
    return target === undefined ? undefined : { ...target, scenePoint: target.scenePoint.clone() };
  }

  public pickFromClientPoint(
    clientX: number,
    clientY: number,
    camera: THREE.Camera,
    element: HTMLElement,
  ): EntitySelectionTarget | undefined {
    const rect = element.getBoundingClientRect();
    if (rect.width <= 0 || rect.height <= 0) return undefined;
    this.pointer.x = ((clientX - rect.left) / rect.width) * 2 - 1;
    this.pointer.y = -((clientY - rect.top) / rect.height) * 2 + 1;
    this.raycaster.setFromCamera(this.pointer, camera);

    for (const intersection of this.raycaster.intersectObjects(this.pickGroup.children, true)) {
      const key = intersection.object.userData.selectionKey;
      if (typeof key !== "string") continue;
      const target = this.targets.get(key);
      if (target !== undefined) return { ...target, scenePoint: target.scenePoint.clone() };
    }
    return undefined;
  }

  public select(target: EntitySelectionTarget | undefined): void {
    this.clearHighlight();
    this.selectedKey = target === undefined ? undefined : selectionKey(target.kind, target.id);
    if (target !== undefined && this.transform !== undefined) this.renderHighlight(target);
  }

  public selectedTarget(): EntitySelectionTarget | undefined {
    if (this.selectedKey === undefined) return undefined;
    const target = this.targets.get(this.selectedKey);
    return target === undefined ? undefined : { ...target, scenePoint: target.scenePoint.clone() };
  }

  public dispose(): void {
    this.clearPickTargets();
    this.clearHighlight();
    this.targets.clear();
    this.regions.clear();
    this.proxies.clear();
    this.pickGroup.removeFromParent();
    this.highlightGroup.removeFromParent();
  }

  private registerRegion(region: RenderRegionSnapshot): string | undefined {
    if (this.transform === undefined || region.boundary.length < 3) return undefined;
    const focus = this.transform.worldPointToScene(region.center.xM, region.center.zM, 0.18);
    const target: EntitySelectionTarget = { kind: "region", id: region.id, scenePoint: focus };
    const key = selectionKey(target.kind, target.id);
    this.targets.set(key, target);
    this.regions.set(region.id, region);

    const signature = region.boundary.map((point) => `${point.xM},${point.zM}`).join(";");
    const existing = this.proxies.get(key);
    if (existing?.userData.signature === signature) return key;
    if (existing !== undefined) {
      disposeObject(existing);
      this.proxies.delete(key);
    }

    const shape = new THREE.Shape();
    const first = this.transform.worldPointToScene(region.boundary[0].xM, region.boundary[0].zM);
    shape.moveTo(first.x, -first.z);
    for (const point of region.boundary.slice(1)) {
      const scenePoint = this.transform.worldPointToScene(point.xM, point.zM);
      shape.lineTo(scenePoint.x, -scenePoint.z);
    }
    shape.closePath();
    const geometry = new THREE.ShapeGeometry(shape);
    const material = new THREE.MeshBasicMaterial({ side: THREE.DoubleSide, transparent: true, opacity: 0, depthWrite: false });
    material.colorWrite = false;
    const mesh = new THREE.Mesh(geometry, material);
    mesh.rotation.x = -Math.PI / 2;
    mesh.position.y = 0.12;
    mesh.name = `region-hit-${region.id}`;
    mesh.userData.selectionKey = key;
    mesh.userData.signature = signature;
    this.pickGroup.add(mesh);
    this.proxies.set(key, mesh);
    return key;
  }

  private registerHumanGroup(group: RenderHumanGroupSnapshot, world: RenderWorldSnapshot): string | undefined {
    if (this.transform === undefined) return undefined;
    const sceneY = sceneHeightAt(world, this.transform, group.xM, group.zM) + 0.35;
    const point = this.transform.worldPointToScene(group.xM, group.zM, sceneY);
    const target: EntitySelectionTarget = { kind: "human_group", id: group.id, scenePoint: point };
    const key = selectionKey(target.kind, target.id);
    this.targets.set(key, target);

    const existing = this.proxies.get(key);
    if (existing instanceof THREE.Mesh) {
      existing.position.copy(point);
      return key;
    }
    if (existing !== undefined) disposeObject(existing);
    const geometry = new THREE.SphereGeometry(0.75, 10, 8);
    const material = new THREE.MeshBasicMaterial({ transparent: true, opacity: 0, depthWrite: false });
    material.colorWrite = false;
    const proxy = new THREE.Mesh(geometry, material);
    proxy.position.copy(point);
    proxy.name = `human-group-hit-${group.id}`;
    proxy.userData.selectionKey = key;
    this.pickGroup.add(proxy);
    this.proxies.set(key, proxy);
    return key;
  }

  private renderHighlight(target: EntitySelectionTarget): void {
    if (target.kind === "region") {
      const region = this.regions.get(target.id);
      if (region !== undefined) {
        this.highlightGroup.add(this.createRegionHighlight(region));
        return;
      }
    }
    this.highlightGroup.add(this.createPointReticle(target.scenePoint));
  }

  private createRegionHighlight(region: RenderRegionSnapshot): THREE.LineLoop {
    if (this.transform === undefined) throw new Error("selection transform unavailable");
    const points = region.boundary.map((point) =>
      this.transform!.worldPointToScene(point.xM, point.zM, 0.22),
    );
    const geometry = new THREE.BufferGeometry().setFromPoints(points);
    const material = new THREE.LineBasicMaterial({ color: 0xffffff, depthTest: false });
    const line = new THREE.LineLoop(geometry, material);
    line.name = `selected-region-${region.id}`;
    line.renderOrder = 20;
    return line;
  }

  private createPointReticle(point: THREE.Vector3): THREE.Mesh {
    const geometry = new THREE.RingGeometry(0.8, 1.05, 28);
    const material = new THREE.MeshBasicMaterial({ color: 0xffffff, side: THREE.DoubleSide, depthTest: false });
    const ring = new THREE.Mesh(geometry, material);
    ring.rotation.x = -Math.PI / 2;
    ring.position.copy(point);
    ring.name = "selected-entity-reticle";
    ring.renderOrder = 20;
    return ring;
  }

  private clearPickTargets(): void {
    for (const proxy of this.proxies.values()) disposeObject(proxy);
    this.proxies.clear();
  }

  private clearHighlight(): void {
    for (const child of [...this.highlightGroup.children]) disposeObject(child);
  }
}

export function selectionKey(kind: SelectableEntityKind, id: string): string {
  return `${kind}:${id}`;
}

function sceneHeightAt(world: RenderWorldSnapshot, transform: WorldSpaceTransform, xM: number, zM: number): number {
  const terrain = world.terrain;
  if (terrain === null || terrain.spacingM <= 0) return 0;
  const x = Math.round((xM - transform.bounds.minXM) / terrain.spacingM);
  const z = Math.round((zM - transform.bounds.minZM) / terrain.spacingM);
  if (x < 0 || z < 0 || x >= terrain.width || z >= terrain.height) return 0;
  return transform.elevationToSceneY(terrain.elevationM[z * terrain.width + x] ?? 0);
}

function disposeObject(object: THREE.Object3D): void {
  object.removeFromParent();
  if (object instanceof THREE.Mesh || object instanceof THREE.Line) {
    object.geometry.dispose();
    const material = object.material;
    if (Array.isArray(material)) {
      for (const item of material) item.dispose();
    } else {
      material.dispose();
    }
  }
}
