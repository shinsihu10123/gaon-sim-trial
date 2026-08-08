import * as THREE from "three";
import type { RenderRegionSnapshot, RenderWorldSnapshot } from "./types";
import { WorldSpaceTransform } from "./world-space";

type RegionVisual = {
  line: THREE.LineLoop;
  signature: string;
};

export class WorldTopologyLayer {
  private readonly group = new THREE.Group();
  private readonly visuals = new Map<string, RegionVisual>();
  private boundsKey = "";

  public constructor(scene: THREE.Scene) {
    this.group.name = "world-topology";
    scene.add(this.group);
  }

  public update(world: RenderWorldSnapshot): void {
    if (!world.initialized || world.bounds === null) {
      this.clear();
      return;
    }

    const nextBoundsKey = `${world.bounds.minXM}:${world.bounds.maxXM}:${world.bounds.minZM}:${world.bounds.maxZM}`;
    const transform = new WorldSpaceTransform(world.bounds);
    const active = new Set<string>();

    for (const region of world.regions) {
      active.add(region.id);
      const signature = regionSignature(region, nextBoundsKey);
      const existing = this.visuals.get(region.id);
      if (existing?.signature === signature) {
        continue;
      }
      if (existing !== undefined) {
        disposeLine(existing.line);
        this.visuals.delete(region.id);
      }
      const line = this.createBoundary(region, transform);
      this.group.add(line);
      this.visuals.set(region.id, { line, signature });
    }

    for (const [id, visual] of [...this.visuals.entries()]) {
      if (!active.has(id)) {
        disposeLine(visual.line);
        this.visuals.delete(id);
      }
    }
    this.boundsKey = nextBoundsKey;
  }

  public dispose(): void {
    this.clear();
    this.group.removeFromParent();
  }

  private createBoundary(
    region: RenderRegionSnapshot,
    transform: WorldSpaceTransform,
  ): THREE.LineLoop {
    const points = region.boundary.map((point) =>
      transform.worldPointToScene(point.xM, point.zM, 0.08),
    );
    const geometry = new THREE.BufferGeometry().setFromPoints(points);
    const material = new THREE.LineBasicMaterial({
      color: region.surface === "ocean" ? 0x777777 : 0xd8d8d8,
    });
    const line = new THREE.LineLoop(geometry, material);
    line.name = `region-${region.id}`;
    line.userData.regionId = region.id;
    return line;
  }

  private clear(): void {
    for (const visual of this.visuals.values()) {
      disposeLine(visual.line);
    }
    this.visuals.clear();
    this.boundsKey = "";
  }
}

function regionSignature(region: RenderRegionSnapshot, boundsKey: string): string {
  return `${boundsKey}|${region.surface}|${region.boundary
    .map((point) => `${point.xM},${point.zM}`)
    .join(";")}`;
}

function disposeLine(line: THREE.Line): void {
  line.removeFromParent();
  line.geometry.dispose();
  const material = line.material;
  if (Array.isArray(material)) {
    for (const item of material) item.dispose();
  } else {
    material.dispose();
  }
}
