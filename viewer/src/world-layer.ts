import * as THREE from "three";
import type { RenderRegionSnapshot, RenderWorldSnapshot } from "./types";

export class WorldTopologyLayer {
  private readonly group = new THREE.Group();

  public constructor(scene: THREE.Scene) {
    this.group.name = "world-topology";
    scene.add(this.group);
  }

  public update(world: RenderWorldSnapshot): void {
    this.clear();
    if (!world.initialized || world.bounds === null) {
      return;
    }

    const centerX = (world.bounds.minXM + world.bounds.maxXM) / 2;
    const centerZ = (world.bounds.minZM + world.bounds.maxZM) / 2;
    const extentX = world.bounds.maxXM - world.bounds.minXM;
    const extentZ = world.bounds.maxZM - world.bounds.minZM;
    const scale = 80 / Math.max(extentX, extentZ, 1);

    for (const region of world.regions) {
      this.group.add(this.createBoundary(region, centerX, centerZ, scale));
    }
  }

  public dispose(): void {
    this.clear();
    this.group.removeFromParent();
  }

  private createBoundary(
    region: RenderRegionSnapshot,
    centerX: number,
    centerZ: number,
    scale: number,
  ): THREE.LineLoop {
    const points = region.boundary.map(
      (point) =>
        new THREE.Vector3((point.xM - centerX) * scale, 0.08, (point.zM - centerZ) * scale),
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
    for (const child of [...this.group.children]) {
      child.removeFromParent();
      if (child instanceof THREE.Line) {
        child.geometry.dispose();
        const material = child.material;
        if (Array.isArray(material)) {
          for (const item of material) {
            item.dispose();
          }
        } else {
          material.dispose();
        }
      }
    }
  }
}
