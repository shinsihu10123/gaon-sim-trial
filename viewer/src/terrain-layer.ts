import * as THREE from "three";
import type { RenderTerrainSnapshot, RenderWorldBounds } from "./types";
import { WorldSpaceTransform } from "./world-space";

const NO_DOWNSTREAM_INDEX = 0xffffffff;

export class TerrainLayer {
  private terrainMesh: THREE.Mesh | undefined;
  private seaMesh: THREE.Mesh | undefined;
  private riverLines: THREE.LineSegments | undefined;
  private renderedTerrain: RenderTerrainSnapshot | null = null;
  private renderedBoundsKey = "";

  public constructor(private readonly scene: THREE.Scene) {}

  public update(bounds: RenderWorldBounds | null, terrain: RenderTerrainSnapshot | null): void {
    if (bounds === null || terrain === null) {
      if (this.renderedTerrain !== null) {
        this.clearMeshes();
        this.renderedTerrain = null;
        this.renderedBoundsKey = "";
      }
      return;
    }

    const boundsKey = `${bounds.minXM}:${bounds.maxXM}:${bounds.minZM}:${bounds.maxZM}`;
    if (this.renderedTerrain === terrain && this.renderedBoundsKey === boundsKey) {
      return;
    }

    this.clearMeshes();
    const sampleCount = terrain.width * terrain.height;
    if (
      terrain.elevationM.length !== sampleCount ||
      terrain.moisturePermille.length !== sampleCount ||
      terrain.biomeCodes.length !== sampleCount ||
      terrain.reliefCodes.length !== sampleCount ||
      terrain.downstreamIndices.length !== sampleCount ||
      terrain.drainageBasinIds.length !== sampleCount ||
      terrain.flowAccumulation.length !== sampleCount ||
      terrain.riverOrders.length !== sampleCount ||
      terrain.landmassIds.length !== sampleCount
    ) {
      throw new Error("terrain snapshot arrays do not match grid dimensions");
    }

    const transform = new WorldSpaceTransform(bounds);
    const positions = new Float32Array(sampleCount * 3);
    const colors = new Float32Array(sampleCount * 3);

    for (let z = 0; z < terrain.height; z += 1) {
      for (let x = 0; x < terrain.width; x += 1) {
        const index = z * terrain.width + x;
        const offset = index * 3;
        const xM = bounds.minXM + x * terrain.spacingM;
        const zM = bounds.minZM + z * terrain.spacingM;
        const point = transform.worldPointToScene(
          xM,
          zM,
          transform.elevationToSceneY(terrain.elevationM[index] ?? 0),
        );

        positions[offset] = point.x;
        positions[offset + 1] = point.y;
        positions[offset + 2] = point.z;

        const color = terrainColor(
          terrain.biomeCodes[index] ?? 0,
          terrain.reliefCodes[index] ?? 0,
          terrain.moisturePermille[index] ?? 0,
          terrain.elevationM[index] ?? 0,
          terrain.seaLevelM,
        );
        colors[offset] = color.r;
        colors[offset + 1] = color.g;
        colors[offset + 2] = color.b;
      }
    }

    const indices: number[] = [];
    for (let z = 0; z < terrain.height - 1; z += 1) {
      for (let x = 0; x < terrain.width - 1; x += 1) {
        const northWest = z * terrain.width + x;
        const northEast = northWest + 1;
        const southWest = northWest + terrain.width;
        const southEast = southWest + 1;
        indices.push(northWest, southWest, northEast, northEast, southWest, southEast);
      }
    }

    const geometry = new THREE.BufferGeometry();
    geometry.setAttribute("position", new THREE.BufferAttribute(positions, 3));
    geometry.setAttribute("color", new THREE.BufferAttribute(colors, 3));
    geometry.setIndex(indices);
    geometry.computeVertexNormals();

    const material = new THREE.MeshStandardMaterial({
      vertexColors: true,
      roughness: 0.96,
      metalness: 0,
      flatShading: false,
    });
    this.terrainMesh = new THREE.Mesh(geometry, material);
    this.terrainMesh.name = "authoritative-terrain-heightfield";
    this.scene.add(this.terrainMesh);

    const seaGeometry = new THREE.PlaneGeometry(
      transform.extentXM * transform.horizontalScale,
      transform.extentZM * transform.horizontalScale,
      1,
      1,
    );
    const seaMaterial = new THREE.MeshPhysicalMaterial({
      color: 0x315f78,
      transparent: true,
      opacity: 0.68,
      roughness: 0.22,
      metalness: 0.02,
      clearcoat: 0.38,
      clearcoatRoughness: 0.3,
      side: THREE.DoubleSide,
      depthWrite: false,
    });
    this.seaMesh = new THREE.Mesh(seaGeometry, seaMaterial);
    this.seaMesh.rotation.x = -Math.PI / 2;
    this.seaMesh.position.y = transform.elevationToSceneY(terrain.seaLevelM) + 0.018;
    this.seaMesh.name = "authoritative-sea-level";
    this.seaMesh.renderOrder = 2;
    this.scene.add(this.seaMesh);

    this.riverLines = buildRiverLines(terrain, positions);
    if (this.riverLines !== undefined) {
      this.scene.add(this.riverLines);
    }
    this.renderedTerrain = terrain;
    this.renderedBoundsKey = boundsKey;
  }

  public dispose(): void {
    this.clearMeshes();
    this.renderedTerrain = null;
    this.renderedBoundsKey = "";
  }

  private clearMeshes(): void {
    if (this.terrainMesh !== undefined) {
      this.terrainMesh.removeFromParent();
      this.terrainMesh.geometry.dispose();
      disposeMaterial(this.terrainMesh.material);
      this.terrainMesh = undefined;
    }
    if (this.seaMesh !== undefined) {
      this.seaMesh.removeFromParent();
      this.seaMesh.geometry.dispose();
      disposeMaterial(this.seaMesh.material);
      this.seaMesh = undefined;
    }
    if (this.riverLines !== undefined) {
      this.riverLines.removeFromParent();
      this.riverLines.geometry.dispose();
      disposeMaterial(this.riverLines.material);
      this.riverLines = undefined;
    }
  }
}

function buildRiverLines(
  terrain: RenderTerrainSnapshot,
  positions: Float32Array,
): THREE.LineSegments | undefined {
  const segments: number[] = [];
  for (let index = 0; index < terrain.riverOrders.length; index += 1) {
    if (terrain.riverOrders[index] === 0) continue;
    const downstream = terrain.downstreamIndices[index];
    if (
      downstream === NO_DOWNSTREAM_INDEX ||
      !Number.isInteger(downstream) ||
      downstream < 0 ||
      downstream >= terrain.riverOrders.length
    ) continue;

    const sourceOffset = index * 3;
    const targetOffset = downstream * 3;
    segments.push(
      positions[sourceOffset], positions[sourceOffset + 1] + 0.028, positions[sourceOffset + 2],
      positions[targetOffset], positions[targetOffset + 1] + 0.028, positions[targetOffset + 2],
    );
  }
  if (segments.length === 0) return undefined;
  const geometry = new THREE.BufferGeometry();
  geometry.setAttribute("position", new THREE.Float32BufferAttribute(segments, 3));
  const material = new THREE.LineBasicMaterial({
    color: 0x72a8bd,
    transparent: true,
    opacity: 0.72,
    depthWrite: false,
  });
  const lines = new THREE.LineSegments(geometry, material);
  lines.name = "authoritative-river-network";
  lines.renderOrder = 4;
  return lines;
}

function terrainColor(
  biomeCode: number,
  reliefCode: number,
  moisturePermille: number,
  elevationM: number,
  seaLevelM: number,
): THREE.Color {
  const base = new THREE.Color(biomeBaseColor(biomeCode));
  const moisture = THREE.MathUtils.clamp(moisturePermille / 1000, 0, 1);
  const elevationAboveSea = elevationM - seaLevelM;

  if (elevationAboveSea <= 0) {
    const depth = THREE.MathUtils.clamp(-elevationAboveSea / 1800, 0, 1);
    return base.lerp(new THREE.Color(0x183f57), 0.2 + depth * 0.42);
  }

  const reliefLightness = reliefCode >= 3 ? 0.1 : reliefCode === 2 ? 0.045 : 0;
  const altitude = THREE.MathUtils.clamp(elevationAboveSea / 3600, 0, 1);
  const dryTint = new THREE.Color(0xb29c70);
  const wetTint = new THREE.Color(0x426a4b);
  base.lerp(moisture < 0.5 ? dryTint : wetTint, Math.abs(moisture - 0.5) * 0.24);
  base.offsetHSL(0, 0, reliefLightness - altitude * 0.045);

  if (altitude > 0.72) {
    base.lerp(new THREE.Color(0xd5d7d4), (altitude - 0.72) / 0.28 * 0.72);
  }
  return base;
}

function biomeBaseColor(biomeCode: number): number {
  switch (biomeCode) {
    case 0: return 0x2b6079;
    case 1: return 0x77875b;
    case 2: return 0x41634a;
    case 3: return 0x9d8b60;
    case 4: return 0x5d7563;
    case 5: return 0xb8bab7;
    default: return 0x777777;
  }
}

function disposeMaterial(material: THREE.Material | THREE.Material[]): void {
  if (Array.isArray(material)) {
    for (const item of material) item.dispose();
  } else {
    material.dispose();
  }
}
