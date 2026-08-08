import * as THREE from "three";
import type { RenderHumanGroupSnapshot, RenderWorldSnapshot } from "./types";
import { WorldSpaceTransform } from "./world-space";

type HumanGroupVisual = {
  root: THREE.Group;
  body: THREE.Mesh;
  halo: THREE.Mesh;
};

export class HumanGroupLayer {
  private readonly group = new THREE.Group();
  private readonly visuals = new Map<string, HumanGroupVisual>();

  public constructor(scene: THREE.Scene) {
    this.group.name = "human-group-visuals";
    scene.add(this.group);
  }

  public update(world: RenderWorldSnapshot): void {
    if (!world.initialized || world.bounds === null) {
      this.clear();
      return;
    }

    const transform = new WorldSpaceTransform(world.bounds);
    const active = new Set<string>();

    for (const snapshot of world.humanGroups) {
      active.add(snapshot.id);
      let visual = this.visuals.get(snapshot.id);
      if (visual === undefined) {
        visual = this.createVisual(snapshot);
        this.group.add(visual.root);
        this.visuals.set(snapshot.id, visual);
      }
      this.updateVisual(visual, snapshot, world, transform);
    }

    for (const [id, visual] of [...this.visuals.entries()]) {
      if (!active.has(id)) {
        disposeVisual(visual);
        this.visuals.delete(id);
      }
    }
  }

  public dispose(): void {
    this.clear();
    this.group.removeFromParent();
  }

  private createVisual(snapshot: RenderHumanGroupSnapshot): HumanGroupVisual {
    const root = new THREE.Group();
    root.name = `human-group-${snapshot.id}`;

    const bodyGeometry = new THREE.IcosahedronGeometry(0.42, 1);
    const bodyMaterial = new THREE.MeshStandardMaterial({
      color: 0xd5b36a,
      roughness: 0.7,
      metalness: 0,
      emissive: 0x2d2110,
      emissiveIntensity: 0.12,
    });
    const body = new THREE.Mesh(bodyGeometry, bodyMaterial);
    body.castShadow = false;
    body.receiveShadow = false;
    body.name = `human-group-body-${snapshot.id}`;

    const haloGeometry = new THREE.RingGeometry(0.58, 0.72, 32);
    const haloMaterial = new THREE.MeshBasicMaterial({
      color: 0xe3c783,
      transparent: true,
      opacity: 0.42,
      side: THREE.DoubleSide,
      depthWrite: false,
    });
    const halo = new THREE.Mesh(haloGeometry, haloMaterial);
    halo.rotation.x = -Math.PI / 2;
    halo.name = `human-group-halo-${snapshot.id}`;
    halo.renderOrder = 8;

    root.add(body, halo);
    return { root, body, halo };
  }

  private updateVisual(
    visual: HumanGroupVisual,
    snapshot: RenderHumanGroupSnapshot,
    world: RenderWorldSnapshot,
    transform: WorldSpaceTransform,
  ): void {
    const groundY = sceneHeightAt(world, transform, snapshot.xM, snapshot.zM);
    const scenePoint = transform.worldPointToScene(snapshot.xM, snapshot.zM, groundY);
    const scale = populationScale(snapshot.population);

    visual.root.position.copy(scenePoint);
    visual.body.position.y = 0.35 * scale;
    visual.body.scale.setScalar(scale);
    visual.halo.position.y = 0.055;
    visual.halo.scale.setScalar(0.9 + scale * 0.28);

    const bodyMaterial = visual.body.material as THREE.MeshStandardMaterial;
    const haloMaterial = visual.halo.material as THREE.MeshBasicMaterial;
    const nutrition = clamp01(snapshot.nutritionPermille / 1000);
    const cohesion = clamp01(snapshot.cohesionPermille / 1000);
    const risk = clamp01(snapshot.riskPermille / 1000);
    const mobility = clamp01(snapshot.mobilityPermille / 1000);
    bodyMaterial.color.setHSL(0.08 + nutrition * 0.05 - risk * 0.035, 0.38 + cohesion * 0.2, 0.48 + nutrition * 0.1);
    haloMaterial.opacity = 0.2 + mobility * 0.18 + cohesion * 0.12;
  }

  private clear(): void {
    for (const visual of this.visuals.values()) disposeVisual(visual);
    this.visuals.clear();
  }
}

function populationScale(populationText: string): number {
  let population = 1;
  try {
    const parsed = BigInt(populationText);
    const capped = parsed > 10_000_000n ? 10_000_000n : parsed;
    population = Number(capped > 0n ? capped : 1n);
  } catch {
    population = 1;
  }
  return THREE.MathUtils.clamp(0.72 + Math.log10(population + 1) * 0.16, 0.8, 1.7);
}

function sceneHeightAt(
  world: RenderWorldSnapshot,
  transform: WorldSpaceTransform,
  xM: number,
  zM: number,
): number {
  const terrain = world.terrain;
  if (terrain === null || terrain.spacingM <= 0) return 0;
  const x = Math.round((xM - transform.bounds.minXM) / terrain.spacingM);
  const z = Math.round((zM - transform.bounds.minZM) / terrain.spacingM);
  if (x < 0 || z < 0 || x >= terrain.width || z >= terrain.height) return 0;
  return transform.elevationToSceneY(terrain.elevationM[z * terrain.width + x] ?? 0);
}

function clamp01(value: number): number {
  return Math.min(1, Math.max(0, value));
}

function disposeVisual(visual: HumanGroupVisual): void {
  visual.root.removeFromParent();
  visual.body.geometry.dispose();
  visual.halo.geometry.dispose();
  disposeMaterial(visual.body.material);
  disposeMaterial(visual.halo.material);
}

function disposeMaterial(material: THREE.Material | THREE.Material[]): void {
  if (Array.isArray(material)) {
    for (const item of material) item.dispose();
  } else {
    material.dispose();
  }
}
