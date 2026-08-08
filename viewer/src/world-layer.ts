import * as THREE from "three";
import type { RenderRegionSnapshot, RenderWorldSnapshot } from "./types";
import { WorldSpaceTransform } from "./world-space";

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

    const transform = new WorldSpaceTransform(world.bounds);
    for (const region of world.regions) {
      this.group.add(this.createBoundary(region, transform));
    }
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
