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
    const color = new THREE.Color();

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

        color.setHex(biomeColor(terrain.biomeCodes[index], terrain.reliefCodes[index]));
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
      roughness: 0.92,
      metalness: 0,
    });
    this.terrainMesh = new THREE.Mesh(geometry, material);
    this.terrainMesh.name = "authoritative-terrain-heightfield";
    this.scene.add(this.terrainMesh);

    const seaGeometry = new THREE.PlaneGeometry(
      transform.extentXM * transform.horizontalScale,
      transform.extentZM * transform.horizontalScale,
    );
    const seaMaterial = new THREE.MeshStandardMaterial({
      color: 0x315b74,
      transparent: true,
      opacity: 0.72,
      roughness: 0.35,
      metalness: 0.05,
      side: THREE.DoubleSide,
    });
    this.seaMesh = new THREE.Mesh(seaGeometry, seaMaterial);
    this.seaMesh.rotation.x = -Math.PI / 2;
    this.seaMesh.position.y = transform.elevationToSceneY(terrain.seaLevelM) + 0.015;
    this.seaMesh.name = "authoritative-sea-level";
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
      positions[sourceOffset], positions[sourceOffset + 1] + 0.025, positions[sourceOffset + 2],
      positions[targetOffset], positions[targetOffset + 1] + 0.025, positions[targetOffset + 2],
    );
  }
  if (segments.length === 0) return undefined;
  const geometry = new THREE.BufferGeometry();
  geometry.setAttribute("position", new THREE.Float32BufferAttribute(segments, 3));
  const material = new THREE.LineBasicMaterial({ color: 0x5f8fa8, transparent: true, opacity: 0.9 });
  const lines = new THREE.LineSegments(geometry, material);
  lines.name = "authoritative-river-network";
  lines.renderOrder = 3;
  return lines;
}

function disposeMaterial(material: THREE.Material | THREE.Material[]): void {
  if (Array.isArray(material)) {
    for (const item of material) item.dispose();
  } else {
    material.dispose();
  }
}

function biomeColor(biomeCode: number, reliefCode: number): number {
  switch (biomeCode) {
    case 0: return reliefCode === 0 ? 0x17364a : 0x27556f;
    case 1: return 0x70835a;
    case 2: return 0x3f6146;
    case 3: return 0xa69468;
    case 4: return 0x586f5e;
    case 5: return 0xaeb0ad;
    default: return 0x777777;
  }
}
